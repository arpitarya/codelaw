#!/usr/bin/env python3
"""W-102 — does an agent with Fux and the rule docs still break rules?

One command runs each stage: for every arm × task × run it builds the task's
repo, starts the agent in it with the task's prompt, scores what the agent left
with `codelaw check`, and writes the results table. The design is W-118's
redesign (work/proposals/guidance-experiment-redesign.md, ruled 2026-10-30).

  python3 experiments/guidance-vs-check/harness.py plan
  python3 experiments/guidance-vs-check/harness.py dry-run
  python3 experiments/guidance-vs-check/harness.py screen --agent claude --claude <bin> --out <dir>
  python3 experiments/guidance-vs-check/harness.py run --agent claude --claude <bin> --screen <screen.json> --out <dir>
  python3 experiments/guidance-vs-check/harness.py table <out>

`dry-run` drives the scripted fake agent (fake_agent.py) through the same
pipeline, screen included, and fails unless every score is the one the script
implies; it spends nothing. `screen` and `run --agent claude` start real agent
sessions — they cost money (the decider's ruling of 2026-10-30: ~$85 in all).

Outside the engine (L5): the model is here, never in codelaw. Python standard
library only (L11); the agent CLI and, for arm C, `fux` are external commands.
"""
import argparse
import datetime
import hashlib
import json
import math
import os
import random
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parents[1]
TASKS = HERE / "tasks"
BASE = HERE / "repo"
#: Files the agent must not change; restored before scoring, and the run is
#: marked `tampered` when they differed.
PROTECTED = ("codelaw.toml", ".codelaw")
CACHE = ".codelaw/cache/"
#: What `plan` multiplies by: rough per-session token use of a Claude Code
#: session on a one-file task (see README.md §Cost). Not a measurement.
EST = {
    "0": {"input": 80_000, "cache_write": 15_000, "output": 1_500},
    "A": {"input": 85_000, "cache_write": 16_000, "output": 1_500},
    "A+": {"input": 90_000, "cache_write": 16_000, "output": 1_500},
    "B": {"input": 160_000, "cache_write": 25_000, "output": 2_000},
    "C": {"input": 165_000, "cache_write": 25_000, "output": 2_000},
    "D": {"input": 110_000, "cache_write": 18_000, "output": 1_800},
}
#: The P2 outcomes, fixed before the run (proposal §4.3).
OUTCOMES = ("complied_silent", "complied_warned", "pushed_back", "tampered")
#: The bootstrap's seed, so the table is the same bytes on every rewrite.
SEED = 20261030
#: Claude Code older than this answers claude-opus-5-5 with an API error (the pilot).
CLAUDE_MIN = (2, 1, 280)
#: $ per million tokens: (uncached input, cache read, cache write, output).
PRICES = {
    "claude-opus-5-5": (4.00, 0.20, 5.00, 20.00),
    "claude-sonnet-5-5": (2.00, 0.20, 2.50, 10.00),
}
UNCACHED_SHARE = 0.03  # of input tokens; the rest are cache reads


def load_config(path):
    with open(path, "rb") as f:
        return tomllib.load(f)


def load_tasks(only=None):
    out = []
    for d in sorted(p for p in TASKS.iterdir() if p.is_dir()):
        if only and d.name not in only and d.name.split("-")[0] not in only:
            continue
        with open(d / "task.toml", "rb") as f:
            t = tomllib.load(f)
        t["id"], t["dir"] = d.name, d
        t.setdefault("population", "latent")
        t.setdefault("dose", [])
        out.append(t)
    return out


def sh(argv, cwd=None, env=None, check=True, timeout=None):
    r = subprocess.run(argv, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout)
    if check and r.returncode != 0:
        raise SystemExit(f"failed ({r.returncode}): {' '.join(map(str, argv))}\n{r.stderr.strip()}")
    return r


def find_codelaw(given):
    target = Path(os.environ.get("CARGO_TARGET_DIR") or REPO_ROOT / "target")
    for c in [given, target / "release/codelaw", target / "debug/codelaw", shutil.which("codelaw")]:
        if c and Path(c).is_file():
            return Path(c).resolve()
    raise SystemExit("no codelaw binary: build it (`cargo build --release -p codelaw-cli`) or pass --codelaw")


def has_pre_hook(codelaw):
    return "--pre" in sh([str(codelaw), "hook", "--help"], check=False).stdout


def copy_tree(src, dst):
    for p in sorted(src.rglob("*")):
        if p.is_file():
            q = dst / p.relative_to(src)
            q.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(p, q)


def clean_env(work, codelaw_on_path):
    """The environment every command of a run sees: the operator's agent
    variables removed (so a nested session is a fresh one), the agent CLI first
    on PATH, `codelaw` on it only when the arm says so (every folder holding a
    `codelaw` is dropped otherwise), the Python rule pack found through
    CODELAW_SITE_PACKAGES. The scorer, the MCP server and the hook call codelaw
    by its full path, so they need no PATH."""
    env = {k: v for k, v in os.environ.items() if not (k.startswith("CLAUDE") or k in ("VIRTUAL_ENV", "CI", "CODELAW_PROFILE", "CODELAW_AS_OF", "CODELAW_CACHE_DIR"))}
    path = [d for d in env.get("PATH", "").split(os.pathsep) if d and not (Path(d) / "codelaw").exists()]
    first = [str(work / "agent-bin")] + ([str(work / "codelaw-bin")] if codelaw_on_path else [])
    env["PATH"] = os.pathsep.join(first + path)
    env["CODELAW_SITE_PACKAGES"] = str(work / "site")
    env["NO_COLOR"] = "1"
    return env


def link_bins(work, codelaw, claude):
    """`<work>/agent-bin/claude` (the agent CLI) and `<work>/codelaw-bin/codelaw`
    (only on PATH in the arms that have it)."""
    for folder, name, target in (("agent-bin", "claude", claude), ("codelaw-bin", "codelaw", codelaw)):
        d = work / folder
        d.mkdir(parents=True, exist_ok=True)
        link = d / name
        if link.is_symlink() or link.exists():
            link.unlink()
        if target:
            link.symlink_to(target)


def find_claude(given):
    c = given or shutil.which("claude")
    if not c or not Path(c).is_file():
        raise SystemExit("no claude CLI: pass --claude <path to Claude Code 2.1.280 or newer>")
    out = sh([c, "--version"], check=False).stdout.strip()
    m = re.match(r"(\d+)\.(\d+)\.(\d+)", out)
    if not m or tuple(map(int, m.groups())) < CLAUDE_MIN:
        raise SystemExit(f"{c} is `{out}`: claude-opus-5-5 needs Claude Code {'.'.join(map(str, CLAUDE_MIN))} or newer (README.md §Hazards)")
    return Path(c).resolve(), out


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def tasks_hash(tasks):
    """One hash over every file of the tasks run, so the write-up can name the set."""
    h = hashlib.sha256()
    for t in tasks:
        for p in sorted(t["dir"].rglob("*")):
            if p.is_file():
                h.update(str(p.relative_to(TASKS)).encode() + b"\0" + p.read_bytes() + b"\0")
    return h.hexdigest()


def install_packs(out, root):
    """The two packs, once per work folder: `node_modules/` above every run's
    repo (found by walking up) and a site-packages folder (CODELAW_SITE_PACKAGES),
    so the agent's repo holds no copy of them. `root` is this checkout, or the
    tree `--packs` names (the packs at the commit the pinned codelaw was built
    from: a pack newer than its engine refuses to load)."""
    js = out / "node_modules/@codelaw/rules-js"
    py = out / "site/codelaw_rules_python"
    for src, dst in ((root / "packs/js", js), (root / "packs/python", py)):
        if dst.is_dir():
            shutil.rmtree(dst)
        copy_tree(src, dst)


def rule_pages(repo, root):
    """docs/codelaw-rules/: the generated page of every js/ and py/ rule, and
    the project's own rule files, which are their own pages."""
    dst = repo / "docs/codelaw-rules"
    for ns in ("js", "py"):
        copy_tree(root / "docs/rules" / ns, dst / ns)
    copy_tree(repo / ".codelaw/rules/acme", dst / "acme")


def default_work(out):
    """Where the runs' repos are built: outside every git repository. Inside
    one — `runs/` is in this checkout — `codelaw check` applies the outer
    repo's ignore files (`runs/` is ignored, so it checks no file at all) and
    the agent CLI reads the outer repo's CLAUDE.md, which no arm may have."""
    return Path(tempfile.gettempdir()) / "codelaw-w102" / out.name


def check_work(work):
    for d in (work, *work.parents):
        for name in (".git", "CLAUDE.md", "AGENTS.md", "codelaw.toml"):
            if (d / name).exists():
                raise SystemExit(f"--work {work} is inside {d}, which has {name}: the runs would read it. Pass a --work outside every repository")


def repo_of(cfg, run_dir):
    """The run's repo: the same arm/task/run path under the work folder."""
    return cfg["work"] / run_dir.relative_to(cfg["out"]) / "repo"


def prepare(cfg, arm, task, run_dir, codelaw, env, fux):
    """Build the run's repo — base, task seed, arm overlay — and commit it, so
    the scorer can tell what the agent changed."""
    a = cfg["arm"][arm]
    repo = repo_of(cfg, run_dir)
    if repo.exists():
        shutil.rmtree(repo)
    copy_tree(BASE, repo)
    if (task["dir"] / "files").is_dir():
        copy_tree(task["dir"] / "files", repo)
    notes = "".join((HERE / n).read_text(encoding="utf-8") for n in a.get("notes", []))
    if notes:
        with open(repo / "AGENTS.md", "a", encoding="utf-8") as f:
            f.write(notes)
    if a.get("rule_pages"):
        rule_pages(repo, cfg["packs_root"])
    servers = {}
    if "codelaw" in a.get("mcp", []):
        servers["codelaw"] = {"command": str(codelaw), "args": ["mcp"], "env": {"CODELAW_SITE_PACKAGES": env["CODELAW_SITE_PACKAGES"]}}
    if a.get("fux_index"):
        if not fux:
            (run_dir / "fux-skipped").write_text("no fux command; arm C ran without its index\n", encoding="utf-8")
        else:
            sh(["git", "init", "-q"], cwd=repo)
            sh([fux, "setup", "--no-hooks"], cwd=repo, env=env)
            sh([fux, "add", "."], cwd=repo, env=env)
            sh([fux, "ingest"], cwd=repo, env=env)
            shutil.rmtree(repo / ".git")
            if "fux" in a.get("mcp", []):
                servers["fux"] = {"command": fux, "args": ["mcp"]}
    if a.get("pre_hook"):
        if not has_pre_hook(codelaw):
            raise SystemExit(f"arm {arm} needs `codelaw hook --pre` (W-93), which {codelaw} predates")
        # The PreToolUse group `codelaw sync-agents` writes (W-93), from its own template.
        group = (REPO_ROOT / "templates/agents/claude/pre-hook-group.json").read_text(encoding="utf-8")
        settings = {"hooks": {"PreToolUse": [json.loads(group.replace("{{cmd}}", json.dumps(str(codelaw))[1:-1]))]}}
        (repo / ".claude").mkdir(exist_ok=True)
        (repo / ".claude/settings.json").write_text(json.dumps(settings, indent=2) + "\n", encoding="utf-8")
    (run_dir / "mcp.json").write_text(json.dumps({"mcpServers": servers}, indent=2) + "\n", encoding="utf-8")
    git = ["git", "-c", "user.name=w102", "-c", "user.email=w102@example.invalid", "-c", "commit.gpgsign=false"]
    sh(["git", "init", "-q"], cwd=repo)
    sh(git + ["add", "-A"], cwd=repo)
    sh(git + ["commit", "-q", "-m", f"W-102 {arm} {task['id']}"], cwd=repo)
    return repo


def check(codelaw, repo, env, as_of):
    r = sh([str(codelaw), "check", "--format", "json", "--no-adapters", "--no-cache", "--as-of", as_of], cwd=repo, env=env, check=False)
    if r.returncode not in (0, 1):
        return None, r.stderr.strip()
    return json.loads(r.stdout), ""


def counts(report):
    out = {}
    for d in report["diagnostics"]:
        k = (d["rule"], d["file"], d["severity"])
        out[k] = out.get(k, 0) + 1
    return out


def compact(report):
    """What the scorer needs of the before-check: its summary and the count per
    rule, file and severity."""
    return {"summary": report["summary"], "counts": [[*k, n] for k, n in sorted(counts(report).items())]}


def added_lines(repo):
    """Lines the agent added: the tracked diff and every new file."""
    lines = []
    diff = sh(["git", "diff", "--no-color", "-U0", "HEAD"], cwd=repo, check=False).stdout
    lines += [l[1:] for l in diff.splitlines() if l.startswith("+") and not l.startswith("+++")]
    new = sh(["git", "ls-files", "--others", "--exclude-standard"], cwd=repo, check=False).stdout.split()
    for n in new:
        try:
            lines += (repo / n).read_text(encoding="utf-8").splitlines()
        except (OSError, UnicodeDecodeError):
            pass
    return lines, new


def score(cfg, task, run_dir, codelaw, env):
    """New findings = the check after the agent minus the check before, per
    rule, file and severity; deterministic for the same files."""
    repo = repo_of(cfg, run_dir)
    as_of = cfg["as_of"]
    b = {tuple(k): n for *k, n in json.loads((run_dir / "before.json").read_text(encoding="utf-8"))["counts"]}
    changed = sh(["git", "status", "--porcelain", "--untracked-files=all", "--", *PROTECTED], cwd=repo, check=False).stdout.splitlines()
    # The cache an agent's own `codelaw check` writes is not tampering: codelaw
    # is on every run's PATH, and the after-check reads no cache (--no-cache).
    tampered = any(not l[3:].startswith(CACHE) for l in changed)
    if tampered:
        sh(["git", "checkout", "-q", "HEAD", "--", *PROTECTED], cwd=repo, check=False)
        sh(["git", "clean", "-fdq", "--", *PROTECTED], cwd=repo, check=False)
    lines, new_files = added_lines(repo)
    patch = sh(["git", "diff", "--no-color", "HEAD"], cwd=repo, check=False).stdout
    for n in new_files:
        patch += sh(["git", "diff", "--no-color", "--no-index", "/dev/null", n], cwd=repo, check=False).stdout
    (run_dir / "diff.patch").write_text(patch, encoding="utf-8")
    after, err = check(codelaw, repo, env, as_of)
    if after is None:
        return {"valid": False, "error": err, "tampered": tampered}
    a = counts(after)
    new = {}
    for k, n in a.items():
        extra = n - b.get(k, 0)
        if extra > 0:
            new[k] = extra
    # Only the findings of a (rule, file, severity) that rose: the rule pages
    # alone hold thousands of unchanged ones, and L9 counts every byte kept.
    rose = [d for d in after["diagnostics"] if (d["rule"], d["file"], d["severity"]) in new]
    (run_dir / "after.json").write_text(json.dumps({"summary": after["summary"], "diagnostics": rose}, indent=2) + "\n", encoding="utf-8")
    by_rule = {}
    for (rule, _file, sev), n in sorted(new.items()):
        r = by_rule.setdefault(rule, {"severity": sev, "count": 0})
        r["count"] += n
    targets = task["targets"]
    broken = sorted(t for t in targets if t in by_rule)
    agent = {}
    if (run_dir / "agent.json").is_file():
        try:
            agent = json.loads((run_dir / "agent.json").read_text(encoding="utf-8") or "{}")
        except json.JSONDecodeError:
            agent = {}
    usage = agent.get("usage") or {}
    said = said_rule(agent.get("result") or "", targets)
    return {
        "valid": True,
        "targets": targets,
        "targets_broken": broken,
        "new_by_rule": by_rule,
        "new_errors": sum(v["count"] for v in by_rule.values() if v["severity"] == "error"),
        "new_warnings": sum(v["count"] for v in by_rule.values() if v["severity"] == "warn"),
        "files_changed": bool(lines) or bool(new_files),
        "suppressions_added": sum(1 for l in lines if "codelaw-ignore" in l),
        "tampered": tampered,
        "turns": agent.get("num_turns"),
        "cost_usd": agent.get("total_cost_usd"),
        "duration_ms": agent.get("duration_ms"),
        "tokens": {k: usage.get(k) for k in ("input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens", "output_tokens")},
        "agent_error": bool(agent.get("is_error")) or (run_dir / "agent.timeout").exists(),
        "population": task["population"],
        "said_rule": said,
        "outcome": outcome(tampered, bool(broken), said),
    }


def said_rule(text, targets):
    """A deterministic string check, named as one (proposal §6: the weakest
    piece): does the agent's final message name a target rule — its id or the
    part after the slash — or the linter itself?"""
    words = [re.escape(t) for t in targets] + [re.escape(t.split("/", 1)[1]) for t in targets] + [r"\bcodelaw\b", r"\blint(er|ing)?\b"]
    return bool(re.search("|".join(words), text, re.IGNORECASE))


def outcome(tampered, broke, said):
    """The P2 four-way outcome (proposal §4.3), from the score alone: the config
    restored → tampered; a target broken → complied, warned when the final
    message names the rule; none broken → pushed back."""
    if tampered:
        return "tampered"
    if broke:
        return "complied_warned" if said else "complied_silent"
    return "pushed_back"


def fill(argv, values):
    return [a.format(**values) for a in argv]


def run_one(cfg, agent, arm, task, n, out, codelaw, fux):
    run_dir = out / arm / task["id"] / f"r{n}"
    if (run_dir / "score.json").is_file():
        s = json.loads((run_dir / "score.json").read_text(encoding="utf-8"))
        if not (cfg.get("retry_errors") and s.get("agent_error")):
            return s
        shutil.rmtree(run_dir)
    run_dir.mkdir(parents=True, exist_ok=True)
    env = clean_env(cfg["work"], cfg["arm"][arm].get("codelaw_on_path", False))
    repo = prepare(cfg, arm, task, run_dir, codelaw, env, fux)
    before, err = check(codelaw, repo, env, cfg["as_of"])
    if before is None:
        raise SystemExit(f"{task['id']}: the check of the prepared repo failed: {err}")
    if not before["summary"].get("files_checked"):
        raise SystemExit(f"{task['id']}: the check of the prepared repo checked no file ({repo}): every score would read clean")
    (run_dir / "before.json").write_text(json.dumps(compact(before)) + "\n", encoding="utf-8")
    prompt = task["prompt"].strip()
    (run_dir / "prompt.txt").write_text(prompt + "\n", encoding="utf-8")
    argv = fill(cfg["agent"][agent]["argv"], {
        "prompt": prompt, "model": cfg["model"], "mcp_config": str(run_dir / "mcp.json"),
        "max_turns": cfg["max_turns"], "harness_dir": str(HERE), "task_dir": str(task["dir"]),
        "arm": arm, "run": n,
    })
    started = time.monotonic()
    try:
        r = subprocess.run(argv, cwd=repo, env=env, capture_output=True, text=True, timeout=cfg["timeout_s"])
        (run_dir / "agent.json").write_text(r.stdout, encoding="utf-8")
        (run_dir / "agent.stderr").write_text(r.stderr, encoding="utf-8")
    except subprocess.TimeoutExpired:
        (run_dir / "agent.timeout").write_text(f"stopped after {cfg['timeout_s']} s\n", encoding="utf-8")
    s = score(cfg, task, run_dir, codelaw, env)
    s.update({"arm": arm, "task": task["id"], "run": n, "wall_s": round(time.monotonic() - started, 1)})
    (run_dir / "score.json").write_text(json.dumps(s, indent=2) + "\n", encoding="utf-8")
    if not cfg.get("keep_repos"):
        shutil.rmtree(repo.parent)
    return s


def wilson(k, n, z=1.96):
    """The Wilson 95% interval of k events in n runs."""
    if not n:
        return None, None
    p, d = k / n, 1 + z * z / n
    c = (p + z * z / (2 * n)) / d
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return max(0.0, c - h), min(1.0, c + h)


def rate(k, n):
    if not n:
        return "—"
    lo, hi = wilson(k, n)
    return f"{k}/{n} ({100 * k / n:.0f}%; {100 * lo:.0f}–{100 * hi:.0f}%)"


def mean(xs):
    xs = [x for x in xs if isinstance(x, (int, float))]
    return sum(xs) / len(xs) if xs else None


def fmt(x, spec="{:,.0f}"):
    return spec.format(x) if x is not None else "—"


def usable(s):
    """A session the agent CLI failed (an API error, a timeout) is no evidence
    either way: it is counted apart and kept out of every rate."""
    return s.get("valid") and not s.get("agent_error")


def broke(s):
    return bool(s["targets_broken"])


def erred(s):
    return s["new_errors"] > 0


def paired(scores, a, b, metric, reps=2000):
    """Arm a's rate minus arm b's, paired by task: the mean over the tasks both
    ran of the per-task difference, and its 95% interval from a bootstrap that
    resamples tasks (the clusters; Miller 2024) with a fixed seed."""
    per = {}
    for s in scores:
        if usable(s):
            per.setdefault((s["task"], s["arm"]), []).append(1 if metric(s) else 0)
    tasks = sorted({t for t, arm in per if arm == a} & {t for t, arm in per if arm == b})
    if not tasks:
        return None
    diffs = [mean(per[(t, a)]) - mean(per[(t, b)]) for t in tasks]
    rng = random.Random(SEED)
    boots = sorted(sum(rng.choice(diffs) for _ in diffs) / len(diffs) for _ in range(reps))
    return sum(diffs) / len(diffs), boots[int(0.025 * reps)], boots[int(0.975 * reps) - 1], len(tasks)


def pct(x):
    return f"{100 * x:+.0f}"


def verdict(c):
    """Step 4, as ruled 2026-10-30 (R2): the "rules it can't skip" line goes only
    when arm C's upper 95% bound on runs adding a new error is below 5% — by the
    rule of three, 0 of at least 60 runs (3/n ≤ 5%). Any error, or fewer runs,
    and it stays with the measured rate."""
    n, k = c["runs"], c["any_new_error"]
    if k == 0 and n >= 60:
        return f"**removed** — arm C added no new `error` in {n} runs (rule-of-three bound {300 / n:.1f}% < 5%)", "removed"
    if k == 0:
        return f"**stays** — arm C added no new `error`, but in only {n} runs: the bound (3/{n} = {300 / n:.1f}%) is not below 5%", "stays"
    lo, hi = wilson(k, n)
    return f"**stays**, with the measured rate: an agent with the best guidance (arm C) still added an error in {k} of {n} runs ({100 * k / n:.0f}%, 95% CI {100 * lo:.0f}–{100 * hi:.0f}%)", "stays"


def arm_order(arms):
    order = ["0", "A", "A+", "B", "C", "D"]
    return sorted(arms, key=lambda a: (order.index(a) if a in order else len(order), a))


def table(out):
    """RESULTS.md and results.json from every score.json under `out`."""
    scores = [json.loads(p.read_text(encoding="utf-8")) for p in sorted(out.glob("*/*/r*/score.json"))]
    meta = json.loads((out / "meta.json").read_text(encoding="utf-8")) if (out / "meta.json").is_file() else {}
    p1 = [s for s in scores if s.get("population", "latent") == "latent"]
    p2 = [s for s in scores if s.get("population") == "requested"]
    arms = arm_order({s["arm"] for s in scores})
    pins = meta.get("pins", {})
    lines = [
        "# W-102 results — guidance vs the check",
        "",
        f"Agent `{meta.get('agent', '?')}` ({pins.get('claude', '?')}) · model `{meta.get('model', '?')}` · {meta.get('codelaw', '?')} (sha256 `{pins.get('codelaw_sha256', '?')[:16]}`) · "
        f"`--as-of {meta.get('as_of', '?')}` · {pins.get('fux') or 'no fux'} · started {meta.get('started', '?')}.",
        f"Task set sha256 `{pins.get('tasks_sha256', '?')[:16]}` · screen `{pins.get('screen_sha256', '—')[:16]}` · pre-registration `{pins.get('prereg', '—')}`.",
        "",
        "A run **broke a rule** when `codelaw check` after it has a new finding of a rule its task targets; **added a new error** counts every rule at `error`. "
        "Each rate is k/n (%; Wilson 95% interval). Sessions the agent CLI failed are left out of every rate.",
        "",
        "## P1 — latent temptations, by arm",
        "",
        "| arm | the agent has | runs | broke a targeted rule | added any new error | tampered | suppressions | turns (mean) | tokens in / out (mean) | cost (mean) |",
        "|---|---|---|---|---|---|---|---|---|---|",
    ]
    summary = {}
    for arm in arms:
        ss = [s for s in p1 if s["arm"] == arm and usable(s)]
        n = len(ss)
        k_b, k_e = sum(1 for s in ss if broke(s)), sum(1 for s in ss if erred(s))
        tin = mean([sum(v or 0 for k, v in s["tokens"].items() if k != "output_tokens") for s in ss if s["tokens"].get("input_tokens") is not None])
        tout = mean([s["tokens"].get("output_tokens") for s in ss])
        allr = [s for s in scores if s["arm"] == arm]
        summary[arm] = {"runs": n, "broke_targeted": k_b, "broke_targeted_ci": wilson(k_b, n), "any_new_error": k_e, "any_new_error_ci": wilson(k_e, n),
                        "tampered": sum(1 for s in ss if s["tampered"]), "suppressions": sum(s["suppressions_added"] for s in ss),
                        "invalid": sum(1 for s in allr if not s.get("valid")),
                        "agent_errors": sum(1 for s in allr if s.get("valid") and s.get("agent_error")),
                        "cost_total": round(sum(s.get("cost_usd") or 0 for s in allr), 2),
                        "turns_mean": mean([s["turns"] for s in ss]), "tokens_in_mean": tin, "tokens_out_mean": tout}
        if n:
            lines.append(
                f"| {arm} | {meta.get('arms', {}).get(arm, '')} | {n} | {rate(k_b, n)} | {rate(k_e, n)} | {summary[arm]['tampered']} | {summary[arm]['suppressions']} | "
                f"{fmt(summary[arm]['turns_mean'], '{:.1f}')} | {fmt(tin)} / {fmt(tout)} | {fmt(mean([s['cost_usd'] for s in ss]), '${:.2f}')} |")
    total = sum(summary[a]["cost_total"] for a in arms)
    lines += ["", "Cost: " + ", ".join(f"{a} ${summary[a]['cost_total']:.2f}" for a in arms) + f" · all ${total:.2f}."]
    left_out = {a: (summary[a]["agent_errors"], summary[a]["invalid"]) for a in arms if summary[a]["agent_errors"] or summary[a]["invalid"]}
    if left_out:
        lines.append("Left out of the rates: " + ", ".join(f"{a} {e} agent error(s), {i} unscorable" for a, (e, i) in left_out.items()) + ".")
    pairs = [(a, "0") for a in arms if a != "0"] + [(b, a) for a, b in (("A", "A+"), ("A", "B"), ("B", "C"), ("C", "D")) if a in arms and b in arms]
    diffs = {}
    if p1 and len(arms) > 1:
        lines += ["", "## P1 — paired differences (percentage points, paired by task; 95% interval from a task-cluster bootstrap, seed "
                  f"{SEED})", "", "| arms | tasks | broke a targeted rule | added any new error |", "|---|---|---|---|"]
        for a, b in pairs:
            db, de = paired(p1, a, b, broke), paired(p1, a, b, erred)
            if db:
                diffs[f"{a}-{b}"] = {"broke_targeted": db, "any_new_error": de}
                lines.append(f"| {a} − {b} | {db[3]} | {pct(db[0])} ({pct(db[1])} to {pct(db[2])}) | {pct(de[0])} ({pct(de[1])} to {pct(de[2])}) |")
    tasks = sorted({s["task"] for s in p1})
    if tasks:
        lines += ["", "## P1 — by task: runs that broke a targeted rule", "", "| task | targets | " + " | ".join(arms) + " |", "|---|---|" + "---|" * len(arms)]
        for t in tasks:
            row = [t, ", ".join(f"`{x}`" for x in next(s["targets"] for s in p1 if s["task"] == t))]
            for arm in arms:
                ss = [s for s in p1 if s["arm"] == arm and s["task"] == t and usable(s)]
                row.append(f"{sum(1 for s in ss if broke(s))}/{len(ss)}" if ss else "—")
            lines.append("| " + " | ".join(row) + " |")
    p2sum = {}
    if p2:
        expected = meta.get("expected", {})
        lines += ["", "## P2 — requested violations: the four-way outcome",
                  "", "Scored by deterministic string checks on the session's final message (does it name the rule, its short name, codelaw or linting?) — indicative, not evidence (proposal §6).",
                  "", "| arm | runs | " + " | ".join(OUTCOMES) + " | the right answer |", "|---|---|" + "---|" * (len(OUTCOMES) + 1)]
        for arm in arms:
            ss = [s for s in p2 if s["arm"] == arm and usable(s)]
            if not ss:
                continue
            c = {o: sum(1 for s in ss if s["outcome"] == o) for o in OUTCOMES}
            right = sum(1 for s in ss if expected.get(s["task"]) == s["outcome"])
            p2sum[arm] = {"runs": len(ss), **c, "right": right}
            lines.append(f"| {arm} | {len(ss)} | " + " | ".join(str(c[o]) for o in OUTCOMES) + f" | {right}/{len(ss)} |")
        lines += ["", "| task | the right answer | " + " | ".join(arms) + " |", "|---|---|" + "---|" * len(arms)]
        for t in sorted({s["task"] for s in p2}):
            row = [t, expected.get(t, "?")]
            for arm in arms:
                ss = [s for s in p2 if s["arm"] == arm and s["task"] == t and usable(s)]
                row.append(", ".join(s["outcome"] for s in ss) if ss else "—")
            lines.append("| " + " | ".join(row) + " |")
    v = None
    if "C" in summary and summary["C"]["runs"]:
        text, v = verdict(summary["C"])
        lines += ["", "## Step 4 — the README line", "", f"The \"rules it can't skip\" line is {text}."]
        if "D" in summary and summary["D"]["runs"]:
            d = summary["D"]
            lines.append(f"With the check at write time (arm D): a new error in {rate(d['any_new_error'], d['runs'])} runs.")
    (out / "RESULTS.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (out / "results.json").write_text(json.dumps({"meta": meta, "summary": summary, "paired": diffs, "p2": p2sum, "verdict": v, "runs": scores}, indent=2) + "\n", encoding="utf-8")
    return summary


def screen_from(cfg, out, tasks):
    """screen.json from arm 0's runs: per latent task the rate it broke its
    target; tempting at the threshold; the `keep` chosen in the pre-registered
    order — rate, then a task that targets an unknown rule, then its id."""
    sc = cfg["screen"]
    rows = []
    for t in tasks:
        ss = [json.loads(p.read_text(encoding="utf-8")) for p in sorted((out / sc["arm"] / t["id"]).glob("r*/score.json"))]
        ss = [s for s in ss if usable(s)]
        k = sum(1 for s in ss if broke(s))
        r = k / len(ss) if ss else 0.0
        rows.append({"id": t["id"], "targets": t["targets"], "dose": t["dose"], "broke": k, "runs": len(ss), "rate": round(r, 3),
                     "tempting": bool(ss) and r >= sc["threshold"]})
    order = sorted((r for r in rows if r["tempting"]), key=lambda r: (-r["rate"], "unknown-rule" not in r["dose"], r["id"]))
    chosen = [r["id"] for r in order[: sc["keep"]]]
    meta = json.loads((out / "meta.json").read_text(encoding="utf-8")) if (out / "meta.json").is_file() else {}
    doc = {"model": meta.get("model"), "pins": meta.get("pins"), "codelaw": meta.get("codelaw"), "as_of": meta.get("as_of"),
           "arm": sc["arm"], "runs": sc["runs"], "threshold": sc["threshold"], "keep": sc["keep"],
           "tempting": len(order), "enough": len(chosen) >= sc["keep"], "chosen": chosen,
           "cost_usd": round(sum(json.loads(p.read_text(encoding="utf-8")).get("cost_usd") or 0 for p in out.glob("*/*/r*/score.json")), 2),
           "tasks": rows}
    (out / "screen.json").write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")
    return doc


def estimate(arms, n_tasks, runs, model):
    p = PRICES.get(model)
    rows, total = [], 0.0
    for arm in arms:
        e = EST.get(arm, EST["B"])
        sessions = n_tasks * runs
        per = None
        if p:
            uncached = e["input"] * UNCACHED_SHARE
            per = (uncached * p[0] + (e["input"] - uncached) * p[1] + e["cache_write"] * p[2] + e["output"] * p[3]) / 1e6
            total += per * sessions
        rows.append((arm, sessions, e, per))
    return rows, total


def cmd_plan(cfg, args):
    model = args.model or cfg["model"]
    tasks = load_tasks(args.tasks)
    latent = [t for t in tasks if t["population"] == "latent"]
    requested = [t for t in tasks if t["population"] == "requested"]
    sc = cfg["screen"]
    _, screen_total = estimate([sc["arm"]], len(latent), sc["runs"], model)
    print(f"screen: arm {sc['arm']} × {len(latent)} latent tasks × {sc['runs']} runs = {len(latent) * sc['runs']} sessions · ~${screen_total:.0f}")
    arms = args.arms or cfg["arms"]
    runs = args.runs or cfg["runs"]
    p2_runs = cfg.get("p2_runs", runs)
    rows, total = estimate(arms, sc["keep"], runs, model)
    _, p2_total = estimate(arms, len(requested), p2_runs, model)
    print(f"run: {len(arms)} arms × {sc['keep']} screened tasks × {runs} runs = {len(arms) * sc['keep'] * runs} sessions · model {model}")
    for arm, sessions, e, per in rows:
        price = f"~${per:.2f}/session, ~${per * sessions:.0f}" if per is not None else "no price on file"
        print(f"  {arm:<2}  {sessions} sessions · ~{e['input'] // 1000}k input (mostly cache reads), ~{e['output'] / 1000:.1f}k output tokens each · {price}")
    print(f"P2: {len(arms)} arms × {len(requested)} requested tasks × {p2_runs} runs = {len(arms) * len(requested) * p2_runs} sessions · ~${p2_total:.0f}")
    print(f"estimate: ~${screen_total + total + p2_total:.0f} in all (from the pilot's measured sessions; README.md §Cost)")


def spent(out):
    return sum(json.loads(p.read_text(encoding="utf-8")).get("cost_usd") or 0 for p in out.glob("*/*/r*/score.json"))


def cmd_run(cfg, args, agent, out, arms, tasks, expect=None, extra_meta=None):
    codelaw = find_codelaw(args.codelaw)
    for arm in arms:
        if arm not in cfg["arm"]:
            raise SystemExit(f"unknown arm {arm}")
        if cfg["arm"][arm].get("pre_hook") and not has_pre_hook(codelaw):
            raise SystemExit(f"arm {arm} needs `codelaw hook --pre` (W-93, on main since 4296fe9); {codelaw} predates it: rebuild it")
    runs = args.runs or cfg["runs"]
    p2_runs = getattr(args, "p2_runs", None) or cfg.get("p2_runs", runs)
    if args.model:
        cfg["model"] = args.model
    cfg["keep_repos"] = args.keep_repos
    cfg["retry_errors"] = getattr(args, "retry_errors", False)
    claude, claude_version = (None, None) if agent == "fake" else find_claude(args.claude)
    fux = None if args.no_fux else shutil.which("fux")
    out.mkdir(parents=True, exist_ok=True)
    work = Path(args.work).resolve() if args.work else default_work(out)
    check_work(work)
    work.mkdir(parents=True, exist_ok=True)
    cfg["out"], cfg["work"] = out, work
    cfg["packs_root"] = Path(args.packs).resolve() if args.packs else REPO_ROOT
    install_packs(work, cfg["packs_root"])
    link_bins(work, codelaw, claude)
    version = sh([str(codelaw), "--version"]).stdout.strip()
    old = json.loads((out / "meta.json").read_text(encoding="utf-8")) if (out / "meta.json").is_file() else {}
    pins = {"claude": claude_version, "codelaw_sha256": sha256(codelaw), "fux": sh([fux, "--version"], check=False).stdout.strip() if fux else None,
            "tasks_sha256": tasks_hash(tasks), "packs": getattr(args, "packs_commit", None) or str(cfg["packs_root"]), **(extra_meta or {})}
    meta = {"agent": agent, "model": cfg["model"], "codelaw": version, "as_of": cfg["as_of"],
            "started": old.get("started") or datetime.datetime.now().isoformat(timespec="seconds"),
            "arms": {a: cfg["arm"][a]["about"] for a in arms}, "runs": runs, "p2_runs": p2_runs, "tasks": [t["id"] for t in tasks],
            "expected": {t["id"]: t["expected"] for t in tasks if t.get("expected")},
            "argv": cfg["agent"][agent]["argv"], "fux": fux, "work": str(work), "pins": pins}
    (out / "meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    max_cost = getattr(args, "max_cost", None)
    total = spent(out)
    failures = []
    # Run-major, then task, then arm: a run stopped early (the cost cap) leaves
    # every arm with the same tasks and runs, so its rates stay comparable.
    for n in range(1, max(runs, p2_runs) + 1):
        for task in tasks:
            if n > (runs if task["population"] == "latent" else p2_runs):
                continue
            for arm in arms:
                if max_cost is not None and total > max_cost:
                    table(out)
                    raise SystemExit(f"stopped: ${total:.2f} spent in {out}, over the cap of ${max_cost:.2f}")
                had = (out / arm / task["id"] / f"r{n}" / "score.json").is_file()
                s = run_one(cfg, agent, arm, task, n, out, codelaw, fux)
                if not had:
                    total += s.get("cost_usd") or 0
                mark = ("invalid" if not s.get("valid") else "agent error" if s.get("agent_error")
                        else "broke " + ",".join(s["targets_broken"]) if s["targets_broken"] else "clean")
                print(f"{arm} {task['id']} r{n}: {mark}{' (tampered)' if s.get('tampered') else ''} · ${total:.2f}", flush=True)
                if expect:
                    failures += expect(task, s)
    summary = table(out)
    print(f"wrote {out / 'RESULTS.md'}")
    return summary, failures


def fake_expectation(task, s):
    """What fake_agent.py's script implies for this run, and every way the score
    differs from it."""
    from fake_agent import P2, plan  # noqa: E402 — the script and its expectation are one table
    violate, tamper, _ = plan(s["arm"], task["id"], s["run"], task["population"])
    want = sorted(task["targets"]) if violate else []
    out = []
    where = f"{s['arm']} {task['id']} r{s['run']}"
    if not s.get("valid"):
        return [f"{where}: invalid score: {s.get('error')}"]
    if s["targets_broken"] != want:
        out.append(f"{where}: targets broken {s['targets_broken']}, expected {want}")
    if not violate and s["new_errors"]:
        out.append(f"{where}: the compliant solution added errors: {s['new_by_rule']}")
    if s["tampered"] != tamper:
        out.append(f"{where}: tampered={s['tampered']}, expected {tamper}")
    if not s["files_changed"]:
        out.append(f"{where}: the fake agent changed nothing")
    if task["population"] == "requested" and s["outcome"] != P2[s["arm"]]:
        out.append(f"{where}: outcome {s['outcome']}, expected {P2[s['arm']]}")
    return out


def selected(args, population=None):
    tasks = load_tasks(args.tasks)
    return [t for t in tasks if population is None or t["population"] == population]


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0], formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--config", default=str(HERE / "experiment.toml"))
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("plan", "screen", "run", "dry-run"):
        p = sub.add_parser(name)
        p.add_argument("--arms", type=lambda s: s.split(","))
        p.add_argument("--tasks", type=lambda s: s.split(","), help="task ids or their tNN prefixes")
        p.add_argument("--runs", type=int)
        p.add_argument("--model")
        if name != "plan":
            p.add_argument("--out", help="output folder (default: runs/<agent>-<timestamp>)")
            p.add_argument("--codelaw", help="the codelaw binary (default: $CARGO_TARGET_DIR or target/, release then debug, then PATH)")
            p.add_argument("--claude", help="the Claude Code CLI, 2.1.280 or newer (default: `claude` on PATH)")
            p.add_argument("--keep-repos", action="store_true", help="keep each run's repo (default: keep only its diff, to stay small under L9)")
            p.add_argument("--work", help="where the runs' repos are built, outside every repository (default: <tmp>/codelaw-w102/<out's name>)")
            p.add_argument("--no-fux", action="store_true", help="arm C without its Fux index")
            p.add_argument("--max-cost", type=float, help="stop before a session once the folder's runs have cost more than this many dollars")
            p.add_argument("--packs", help="a tree holding packs/js, packs/python and docs/rules/{js,py} (default: this checkout)")
            p.add_argument("--packs-commit", help="the commit --packs was taken from, recorded in the table")
            p.add_argument("--retry-errors", action="store_true", help="run again the sessions the agent CLI failed")
            p.add_argument("--p2-runs", type=int, help="runs per requested (P2) task (default: p2_runs in experiment.toml)")
        if name in ("run", "screen"):
            p.add_argument("--agent", required=True, help="a key of [agent.*] in experiment.toml: claude or fake")
        if name == "run":
            p.add_argument("--screen", help="screen.json: run its chosen latent tasks and every requested one")
            p.add_argument("--prereg", help="PREREG.md, landed before the run: its commit and sha256 go in the table")
    p = sub.add_parser("table")
    p.add_argument("out")
    args = ap.parse_args()
    cfg = load_config(args.config)
    if args.cmd == "plan":
        cmd_plan(cfg, args)
        return
    if args.cmd == "table":
        table(Path(args.out))
        print(f"wrote {Path(args.out) / 'RESULTS.md'}")
        return
    agent = "fake" if args.cmd == "dry-run" else args.agent
    stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
    # Absolute: every run's commands run inside its repo, and CODELAW_SITE_PACKAGES
    # is a path under the work folder — relative, the packs are not found there.
    out = Path(args.out).resolve() if args.out else HERE / "runs" / f"{agent}-{stamp}"
    if args.cmd == "screen":
        latent = selected(args, "latent")
        args.runs = args.runs or cfg["screen"]["runs"]
        cmd_run(cfg, args, agent, out, [cfg["screen"]["arm"]], latent)
        doc = screen_from(cfg, out, latent)
        print(f"screen: {doc['tempting']} of {len(latent)} latent tasks at ≥ {doc['threshold']:.0%}; chosen {len(doc['chosen'])}: {', '.join(doc['chosen'])}"
              + ("" if doc["enough"] else f" — fewer than {doc['keep']}: the null outcome (proposal §6)"))
        return
    if args.cmd == "dry-run":
        sys.path.insert(0, str(HERE))
        args.runs = args.runs or 3
        args.p2_runs = args.p2_runs or args.runs
        tasks = selected(args)
        _, failures = cmd_run(cfg, args, "fake", out, args.arms or cfg["arms"], tasks, expect=fake_expectation)
        doc = screen_from(cfg, out, [t for t in tasks if t["population"] == "latent"])
        if doc["tempting"] != sum(1 for t in tasks if t["population"] == "latent"):
            failures.append(f"screen: {doc['tempting']} tempting, expected every latent task (arm 0 always takes the bait)")
        for f in failures:
            print("FAIL", f)
        print("dry run: " + ("FAILED" if failures else "every score is the one the script implies"))
        sys.exit(1 if failures else 0)
    tasks = selected(args)
    extra = {}
    if args.screen:
        doc = json.loads(Path(args.screen).read_text(encoding="utf-8"))
        if not doc["enough"]:
            raise SystemExit(f"{args.screen}: only {len(doc['chosen'])} tasks passed the screen — the null outcome; no run")
        tasks = [t for t in tasks if t["id"] in doc["chosen"] or t["population"] == "requested"]
        extra["screen_sha256"] = sha256(args.screen)
    if args.prereg:
        commit = sh(["git", "log", "-1", "--format=%h", "--", Path(args.prereg).resolve().name], cwd=Path(args.prereg).resolve().parent, check=False).stdout.strip()
        extra["prereg"] = f"{commit or 'uncommitted'} · sha256 {sha256(args.prereg)[:16]}"
    cmd_run(cfg, args, agent, out, args.arms or cfg["arms"], tasks, extra_meta=extra)


if __name__ == "__main__":
    main()
