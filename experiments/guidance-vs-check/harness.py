#!/usr/bin/env python3
"""W-102 — does an agent with Fux and the rule docs still break rules?

One command runs the whole experiment: for every arm × task × run it builds the
task's repo, starts the agent in it with the task's prompt, scores what the
agent left with `codelaw check`, and writes the results table.

  python3 experiments/guidance-vs-check/harness.py plan
  python3 experiments/guidance-vs-check/harness.py run --agent claude
  python3 experiments/guidance-vs-check/harness.py dry-run
  python3 experiments/guidance-vs-check/harness.py table <out>

`dry-run` drives the scripted fake agent (fake_agent.py) through the same
pipeline and fails unless every score is the one the script implies; it spends
nothing. `run --agent claude` starts real agent sessions — it costs money and is
the decider's call (W-102's inbox row).

Outside the engine (L5): the model is here, never in codelaw. Python standard
library only (L11); the agent CLI and, for arm C, `fux` are external commands.
"""
import argparse
import datetime
import json
import os
import shutil
import subprocess
import sys
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
#: What `plan` multiplies by: rough per-session token use of a Claude Code
#: session on a one-file task (see README.md §Cost). Not a measurement.
EST = {
    "A": {"input": 250_000, "cache_write": 30_000, "output": 6_000},
    "B": {"input": 400_000, "cache_write": 45_000, "output": 8_000},
    "C": {"input": 450_000, "cache_write": 50_000, "output": 9_000},
    "D": {"input": 300_000, "cache_write": 30_000, "output": 7_000},
}
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


def clean_env(codelaw, out):
    """The environment every command of a run sees: the operator's agent
    variables removed (so a nested session is a fresh one), codelaw on PATH,
    the Python rule pack found through CODELAW_SITE_PACKAGES."""
    env = {k: v for k, v in os.environ.items() if not (k.startswith("CLAUDE") or k in ("VIRTUAL_ENV", "CI", "CODELAW_PROFILE", "CODELAW_AS_OF", "CODELAW_CACHE_DIR"))}
    env["PATH"] = f"{codelaw.parent}{os.pathsep}{env.get('PATH', '')}"
    env["CODELAW_SITE_PACKAGES"] = str(out / "site")
    env["NO_COLOR"] = "1"
    return env


def install_packs(out):
    """The two packs, once per output folder: `node_modules/` above every run's
    repo (found by walking up) and a site-packages folder (CODELAW_SITE_PACKAGES),
    so the agent's repo holds no copy of them."""
    js = out / "node_modules/@codelaw/rules-js"
    py = out / "site/codelaw_rules_python"
    for src, dst in ((REPO_ROOT / "packs/js", js), (REPO_ROOT / "packs/python", py)):
        if not dst.is_dir():
            copy_tree(src, dst)


def rule_pages(repo):
    """docs/codelaw-rules/: the generated page of every js/ and py/ rule, and
    the project's own rule files, which are their own pages."""
    dst = repo / "docs/codelaw-rules"
    for ns in ("js", "py"):
        copy_tree(REPO_ROOT / "docs/rules" / ns, dst / ns)
    copy_tree(repo / ".codelaw/rules/acme", dst / "acme")


def prepare(cfg, arm, task, run_dir, codelaw, env, fux):
    """Build the run's repo — base, task seed, arm overlay — and commit it, so
    the scorer can tell what the agent changed."""
    a = cfg["arm"][arm]
    repo = run_dir / "repo"
    if repo.exists():
        shutil.rmtree(repo)
    copy_tree(BASE, repo)
    if (task["dir"] / "files").is_dir():
        copy_tree(task["dir"] / "files", repo)
    notes = "".join((HERE / a[k]).read_text(encoding="utf-8") for k in ("notes", "notes_extra") if a.get(k))
    if notes:
        with open(repo / "AGENTS.md", "a", encoding="utf-8") as f:
            f.write(notes)
    if a.get("rule_pages"):
        rule_pages(repo)
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
    repo = run_dir / "repo"
    as_of = cfg["as_of"]
    b = {tuple(k): n for *k, n in json.loads((run_dir / "before.json").read_text(encoding="utf-8"))["counts"]}
    changed = sh(["git", "status", "--porcelain", "--", *PROTECTED], cwd=repo, check=False).stdout.strip()
    tampered = bool(changed)
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
    }


def fill(argv, values):
    return [a.format(**values) for a in argv]


def run_one(cfg, agent, arm, task, n, out, codelaw, env, fux):
    run_dir = out / arm / task["id"] / f"r{n}"
    if (run_dir / "score.json").is_file():
        return json.loads((run_dir / "score.json").read_text(encoding="utf-8"))
    run_dir.mkdir(parents=True, exist_ok=True)
    repo = prepare(cfg, arm, task, run_dir, codelaw, env, fux)
    before, err = check(codelaw, repo, env, cfg["as_of"])
    if before is None:
        raise SystemExit(f"{task['id']}: the check of the prepared repo failed: {err}")
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
        shutil.rmtree(repo)
    return s


def rate(k, n):
    return f"{k}/{n} ({100 * k / n:.0f}%)" if n else "—"


def mean(xs):
    xs = [x for x in xs if isinstance(x, (int, float))]
    return sum(xs) / len(xs) if xs else None


def fmt(x, spec="{:,.0f}"):
    return spec.format(x) if x is not None else "—"


def table(out):
    """RESULTS.md and results.json from every score.json under `out`."""
    scores = [json.loads(p.read_text(encoding="utf-8")) for p in sorted(out.glob("*/*/r*/score.json"))]
    meta = json.loads((out / "meta.json").read_text(encoding="utf-8")) if (out / "meta.json").is_file() else {}
    arms = sorted({s["arm"] for s in scores})
    tasks = sorted({s["task"] for s in scores})
    lines = [
        "# W-102 results — guidance vs the check",
        "",
        f"Agent `{meta.get('agent', '?')}` · model `{meta.get('model', '?')}` · {meta.get('codelaw', '?')} · started {meta.get('started', '?')}.",
        "A run **broke a rule** when `codelaw check` after it has a new finding of a rule its task targets; **new errors** counts every rule. Rates are over the runs scored, never one run.",
        "",
        "| arm | what it adds | runs | broke a targeted rule | added any new error | targeted findings | tampered | suppressions | turns (mean) | tokens in / out (mean) | cost (mean) |",
        "|---|---|---|---|---|---|---|---|---|---|---|",
    ]
    summary = {}
    for arm in arms:
        ss = [s for s in scores if s["arm"] == arm and s.get("valid")]
        n = len(ss)
        broke = sum(1 for s in ss if s["targets_broken"])
        err = sum(1 for s in ss if s["new_errors"])
        tf = sum(s["new_by_rule"].get(t, {}).get("count", 0) for s in ss for t in s["targets"])
        tin = mean([sum(v or 0 for k, v in s["tokens"].items() if k != "output_tokens") for s in ss if s["tokens"].get("input_tokens") is not None])
        tout = mean([s["tokens"].get("output_tokens") for s in ss])
        summary[arm] = {"runs": n, "broke_targeted": broke, "any_new_error": err, "targeted_findings": tf,
                        "tampered": sum(1 for s in ss if s["tampered"]), "suppressions": sum(s["suppressions_added"] for s in ss),
                        "invalid": sum(1 for s in scores if s["arm"] == arm and not s.get("valid"))}
        lines.append(
            f"| {arm} | {meta.get('arms', {}).get(arm, '')} | {n} | {rate(broke, n)} | {rate(err, n)} | {tf} | {summary[arm]['tampered']} | {summary[arm]['suppressions']} | "
            f"{fmt(mean([s['turns'] for s in ss]), '{:.1f}')} | {fmt(tin)} / {fmt(tout)} | {fmt(mean([s['cost_usd'] for s in ss]), '${:.2f}')} |")
    lines += ["", "## By task — runs that broke a targeted rule", "", "| task | targets | " + " | ".join(arms) + " |", "|---|---|" + "---|" * len(arms)]
    for t in tasks:
        row = [t, ", ".join(f"`{x}`" for x in next(s["targets"] for s in scores if s["task"] == t and s.get("valid")) or [])]
        for arm in arms:
            ss = [s for s in scores if s["arm"] == arm and s["task"] == t and s.get("valid")]
            row.append(f"{sum(1 for s in ss if s['targets_broken'])}/{len(ss)}" if ss else "—")
        lines.append("| " + " | ".join(row) + " |")
    if "C" in summary:
        c = summary["C"]
        verdict = ("**removed** — arm C added no new `error` in any run" if c["runs"] and c["any_new_error"] == 0
                   else f"**stays**, with the measured rate: in {c['any_new_error']} of {c['runs']} runs an agent with the best guidance still added an error")
        lines += ["", "## Step 4 — the README line", "", f"The \"rules it can't skip\" line is {verdict}."]
    (out / "RESULTS.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (out / "results.json").write_text(json.dumps({"meta": meta, "summary": summary, "runs": scores}, indent=2) + "\n", encoding="utf-8")
    return summary


def estimate(cfg, arms, n_tasks, runs, model):
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
    arms = args.arms or cfg["arms"]
    tasks = load_tasks(args.tasks)
    runs = args.runs or cfg["runs"]
    model = args.model or cfg["model"]
    rows, total = estimate(cfg, arms, len(tasks), runs, model)
    print(f"{len(arms)} arms × {len(tasks)} tasks × {runs} runs = {len(arms) * len(tasks) * runs} agent sessions · model {model}")
    for arm, sessions, e, per in rows:
        price = f"~${per:.2f}/session, ~${per * sessions:.0f}" if per is not None else "no price on file"
        print(f"  {arm}  {sessions} sessions · ~{e['input'] // 1000}k input (mostly cache reads), ~{e['output'] // 1000}k output tokens each · {price}")
    if total:
        print(f"estimate: ~${total:.0f} (a rough range is half to twice that; README.md §Cost)")


def cmd_run(cfg, args, agent, out, expect=None):
    codelaw = find_codelaw(args.codelaw)
    arms = args.arms or cfg["arms"]
    for arm in arms:
        if arm not in cfg["arm"]:
            raise SystemExit(f"unknown arm {arm}")
        if cfg["arm"][arm].get("pre_hook") and not has_pre_hook(codelaw):
            raise SystemExit(f"arm {arm} needs `codelaw hook --pre` (W-93, on main since 4296fe9); {codelaw} predates it: rebuild it")
    tasks = load_tasks(args.tasks)
    runs = args.runs or cfg["runs"]
    if args.model:
        cfg["model"] = args.model
    cfg["keep_repos"] = args.keep_repos
    fux = None if args.no_fux else shutil.which("fux")
    out.mkdir(parents=True, exist_ok=True)
    install_packs(out)
    env = clean_env(codelaw, out)
    version = sh([str(codelaw), "--version"]).stdout.strip()
    meta = {"agent": agent, "model": cfg["model"], "codelaw": version, "started": datetime.datetime.now().isoformat(timespec="seconds"),
            "arms": {a: cfg["arm"][a]["about"] for a in arms}, "runs": runs, "tasks": [t["id"] for t in tasks],
            "argv": cfg["agent"][agent]["argv"], "fux": fux}
    (out / "meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    failures = []
    for arm in arms:
        for task in tasks:
            for n in range(1, runs + 1):
                s = run_one(cfg, agent, arm, task, n, out, codelaw, env, fux)
                mark = "broke " + ",".join(s["targets_broken"]) if s.get("targets_broken") else ("invalid" if not s.get("valid") else "clean")
                print(f"{arm} {task['id']} r{n}: {mark}{' (tampered)' if s.get('tampered') else ''}", flush=True)
                if expect:
                    failures += expect(task, s)
    summary = table(out)
    print(f"wrote {out / 'RESULTS.md'}")
    return summary, failures


def fake_expectation(task, s):
    """What fake_agent.py's script implies for this run, and every way the score
    differs from it."""
    from fake_agent import plan  # noqa: E402 — the script and its expectation are one table
    violate, tamper = plan(s["arm"], task["id"], s["run"])
    want = sorted(task["targets"]) if violate else []
    out = []
    if not s.get("valid"):
        return [f"{s['arm']} {task['id']} r{s['run']}: invalid score: {s.get('error')}"]
    if s["targets_broken"] != want:
        out.append(f"{s['arm']} {task['id']} r{s['run']}: targets broken {s['targets_broken']}, expected {want}")
    if not violate and s["new_errors"]:
        out.append(f"{s['arm']} {task['id']} r{s['run']}: the compliant solution added errors: {s['new_by_rule']}")
    if s["tampered"] != tamper:
        out.append(f"{s['arm']} {task['id']} r{s['run']}: tampered={s['tampered']}, expected {tamper}")
    if not s["files_changed"]:
        out.append(f"{s['arm']} {task['id']} r{s['run']}: the fake agent changed nothing")
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0], formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--config", default=str(HERE / "experiment.toml"))
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("plan", "run", "dry-run"):
        p = sub.add_parser(name)
        p.add_argument("--arms", type=lambda s: s.split(","))
        p.add_argument("--tasks", type=lambda s: s.split(","), help="task ids or their tNN prefixes")
        p.add_argument("--runs", type=int)
        p.add_argument("--model")
        if name != "plan":
            p.add_argument("--out", help="output folder (default: runs/<agent>-<timestamp>)")
            p.add_argument("--codelaw", help="the codelaw binary (default: $CARGO_TARGET_DIR or target/, release then debug, then PATH)")
            p.add_argument("--keep-repos", action="store_true", help="keep each run's repo (default: keep only its diff, to stay small under L9)")
            p.add_argument("--no-fux", action="store_true", help="arm C without its Fux index")
        if name == "run":
            p.add_argument("--agent", required=True, help="a key of [agent.*] in experiment.toml: claude or fake")
    p = sub.add_parser("table")
    p.add_argument("out")
    args = ap.parse_args()
    cfg = load_config(args.config)
    if args.cmd == "plan":
        cmd_plan(cfg, args)
    elif args.cmd == "table":
        table(Path(args.out))
        print(f"wrote {Path(args.out) / 'RESULTS.md'}")
    else:
        agent = "fake" if args.cmd == "dry-run" else args.agent
        stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
        out = Path(args.out) if args.out else HERE / "runs" / f"{agent}-{stamp}"
        if args.cmd == "dry-run":
            sys.path.insert(0, str(HERE))
            _, failures = cmd_run(cfg, args, "fake", out, expect=fake_expectation)
            for f in failures:
                print("FAIL", f)
            print("dry run: " + ("FAILED" if failures else "every score is the one the script implies"))
            sys.exit(1 if failures else 0)
        cmd_run(cfg, args, agent, out)


if __name__ == "__main__":
    main()
