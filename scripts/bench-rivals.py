#!/usr/bin/env python3
"""The side-by-side bench (W-99): codelaw beside ESLint, Prettier, Biome,
oxlint and Ruff on L8 decision 2's three pinned repos, on one machine.

    bench-rivals.py --self-test          the table is well-formed (no network)
    bench-rivals.py plan                 each tool, its pin, licence and L11 state
    bench-rivals.py gate [--github-output F]
                                         run=true when at least one rival is ratified
    bench-rivals.py install --tools D    install every ratified rival at its pin
    bench-rivals.py run --binary B [--corpus D] [--tools D] [--only T,...]
                        [--repo NAME ...] [--runs N] [--out D]
                                         time each tool; write results.json and results.md

perf/rivals.toml is the table: per tool its exact version, its licence, its
row in L11's list, how it is installed, the config it runs with and the
command for lint and for format. The method is `codelaw bench`'s
(SR-CLI-MCP decision 10): `--runs` runs (10 by default), the first
discarded, the median and nearest-rank p95 of the rest, spawn to exit.
Cold is a fresh, empty cache folder per run; warm is a cache primed by one
unrecorded run, still a process of its own (no daemon: the rivals have
none). A tool with no cache is reported cold only.

Each tool gets its own copy of each repo — without the repo's own configs
for any of these tools (`[bench] strip`) — with the tool's config written at
its root, so every tool runs its recommended preset on the same files.
codelaw's packs are installed beside the copies, as a user's npm and Python
installs hold them. Because each tool covers different languages, codelaw is
also timed with `--lang` set to each rival's languages: compare a rival with
that row.

NO RIVAL RUNS BEFORE THE DECIDER RATIFIES IT (L11). `install` and `run`
refuse a tool whose row (`l11`) is not in L11's list as `ratified …` at the
table's version; that list, not this script or CI, is the switch. Until
then `run` times codelaw alone — the dry run — and `gate` tells
bench-rivals.yml to skip. Numbers from here are context rows for L8
§Measurements, never a budget. The standard library only, Python 3.11.
"""
import argparse
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import time
import tomllib
from datetime import datetime, timezone
from pathlib import Path

for _s in (sys.stdout, sys.stderr):
    if hasattr(_s, "reconfigure"):
        _s.reconfigure(encoding="utf-8")

ROOT = Path(__file__).resolve().parent.parent
TABLE = ROOT / "perf" / "rivals.toml"
L11 = ROOT / "records" / "0013_LAW-11-dependencies.md"
PACKS = ROOT / "packs"
OPS = ("lint", "format")
# codelaw's `--lang` names (crates/codelaw-lang), for the table's self-test.
LANGS = {"javascript", "jsx", "typescript", "tsx", "python", "json", "yaml", "toml", "markdown", "css", "scss", "html"}


def die(msg):
    print(f"bench-rivals: {msg}", file=sys.stderr)
    sys.exit(2)


def load():
    with TABLE.open("rb") as f:
        return tomllib.load(f)


# --------------------------------------------------------------- L11's list

def l11_state(tool):
    """`own`, `ratified` or why not, read from L11's list (§3) — the only
    switch that lets a rival be installed or run."""
    row = tool["l11"]
    if row == "own":
        return "own", "codelaw's own build"
    line = next((l for l in L11.read_text(encoding="utf-8").splitlines() if re.match(rf"^\|\s*{re.escape(row)}\s*\|", l)), None)
    if line is None:
        return "absent", f"{row} is not in L11's list (records/0013_LAW-11-dependencies.md §3) — proposed in work/open/W-99-side-by-side-bench.md"
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    status = cells[-1]
    if not status.startswith("ratified"):
        return "unratified", f"{row} is `{status.split(' ')[0]}` in L11's list, not `ratified`"
    if tool["version"] not in line:
        return "other-version", f"{row} is ratified, but not at {tool['version']}: ratify the new pin first (L11 decision 4)"
    return "ratified", status


def runnable(table, only=None):
    """codelaw and every ratified rival, in table order; a rival named in
    `only` that is not ratified is refused, never skipped quietly."""
    out = []
    for name, tool in table["tool"].items():
        if only is not None and name not in only:
            continue
        state, why = l11_state(tool)
        if state in ("own", "ratified"):
            out.append(name)
        elif only is not None:
            die(f"{name}: not run — {why}")
    return out


# --------------------------------------------------------------- self-test

def self_test(table):
    errors = []
    tools = table.get("tool", {})
    if "codelaw" not in tools:
        errors.append("no [tool.codelaw]")
    for name, t in tools.items():
        for key in ("version", "licence", "l11", "rule_set", "languages", "cache", "ok_exit"):
            if key not in t:
                errors.append(f"{name}: no `{key}`")
        if not any(op in t for op in OPS):
            errors.append(f"{name}: neither `lint` nor `format`")
        for op in OPS:
            if op in t and (not t[op] or not t[op][0].startswith("{bin}/")):
                errors.append(f"{name}.{op}: the command starts with {{bin}}/")
            if op in t and not t["cache"] and any("{cache}" in a for a in t[op]):
                errors.append(f"{name}.{op}: names {{cache}} but `cache = false`")
        if ("config" in t) != ("config_file" in t):
            errors.append(f"{name}: `config` and `config_file` go together")
        if name == "codelaw":
            if t.get("l11") != "own":
                errors.append("codelaw: l11 = \"own\"")
            continue
        if not re.fullmatch(r"D-\d+", t.get("l11", "")):
            errors.append(f"{name}: l11 names a row, D-nn")
        inst = t.get("install", {})
        specs = inst.get("npm", []) + inst.get("pypi", [])
        if not specs:
            errors.append(f"{name}: no `install`")
        for spec in specs:
            if not re.fullmatch(r"(@?[\w./-]+)(@|==)\d+\.\d+\.\d+", spec):
                errors.append(f"{name}: `{spec}` is not an exact version")
        if not any(spec.endswith(t.get("version", "?")) for spec in specs):
            errors.append(f"{name}: no install spec is at `version` {t.get('version')}")
        langs = set(t.get("codelaw_lang", "").split(","))
        if not langs <= LANGS:
            errors.append(f"{name}: codelaw_lang names {sorted(langs - LANGS)}, not codelaw languages")
    corpus = {r["name"] for r in json.loads((ROOT / "scripts" / "corpus.json").read_text(encoding="utf-8"))["repos"]}
    for r in table["bench"]["repos"]:
        if r not in corpus:
            errors.append(f"[bench] repos: {r} is not in scripts/corpus.json")
    for e in errors:
        print(f"  ✗ {e}")
    print(f"perf/rivals.toml: {len(tools)} tools, {'ok' if not errors else f'{len(errors)} errors'}")
    return 0 if not errors else 1


# --------------------------------------------------------------- plan, gate

def plan(table):
    print(f"{'tool':<9} {'version':<20} {'L11':<6} {'state':<14} licence")
    for name, t in table["tool"].items():
        state, why = l11_state(t)
        print(f"{name:<9} {t['version']:<20} {t['l11']:<6} {state:<14} {t['licence']}")
        if state not in ("own", "ratified"):
            print(f"{'':<9} ↳ {why}")
    return 0


def gate(table, github_output):
    ratified = [n for n in runnable(table) if n != "codelaw"]
    run = bool(ratified)
    print(f"run={'true' if run else 'false'} — ratified rivals: {', '.join(ratified) or 'none'}")
    if github_output:
        with open(github_output, "a", encoding="utf-8") as f:
            f.write(f"run={'true' if run else 'false'}\n")
    return 0


# --------------------------------------------------------------- install

def install(table, tools_dir, only=None):
    names = runnable(table, only)
    tools_dir.mkdir(parents=True, exist_ok=True)
    trees = {}
    for name in names:
        if name == "codelaw":
            continue
        t, at = table["tool"][name], tools_dir / name
        if at.exists():
            shutil.rmtree(at)
        at.mkdir(parents=True)
        inst = t["install"]
        if inst.get("npm"):
            (at / "package.json").write_text('{"private": true}\n', encoding="utf-8")
            subprocess.run(["npm", "install", "--prefix", str(at), "--save-exact", "--ignore-scripts", "--no-audit", "--no-fund", *inst["npm"]], check=True)
            lock = json.loads((at / "package-lock.json").read_text(encoding="utf-8"))["packages"]
            # what is on disk: of the optional platform binaries, only this machine's
            trees[name] = sorted(f"{k.split('node_modules/')[-1]}@{v.get('version')} {v.get('license', '?')}" for k, v in lock.items() if k and (at / k).is_dir())
        if inst.get("pypi"):
            subprocess.run([sys.executable, "-m", "venv", str(at / "venv")], check=True)
            subprocess.run([str(at / "venv" / "bin" / "pip"), "install", "--quiet", "--only-binary", ":all:", "--no-deps", *inst["pypi"]], check=True)
            trees[name] = list(inst["pypi"])
        print(f"installed {name} {t['version']}: {len(trees[name])} packages")
    (tools_dir / "installed.json").write_text(json.dumps(trees, indent=2) + "\n", encoding="utf-8")
    return 0


def bin_dir(table, name, tools_dir, binary):
    if name == "codelaw":
        return binary.parent
    t = table["tool"][name]
    at = tools_dir / name
    return at / ("venv/bin" if t["install"].get("pypi") else "node_modules/.bin")


# --------------------------------------------------------------- copies

def copy_repo(src, dst, strip):
    """`src` into `dst` without anything named in `strip`; the file count."""
    n = 0
    dst.mkdir(parents=True, exist_ok=True)
    for e in os.scandir(src):
        if e.name in strip:
            continue
        if e.is_dir(follow_symlinks=False):
            n += copy_repo(Path(e.path), dst / e.name, strip)
        elif e.is_file(follow_symlinks=False):
            shutil.copyfile(e.path, dst / e.name)
            n += 1
    return n


def install_packs(work):
    """Every first-party pack where codelaw finds it: npm's under
    `node_modules/` (found walking up from the project), the PyPI-only ones in
    a site-packages folder named by CODELAW_SITE_PACKAGES."""
    for pack in sorted(p for p in PACKS.iterdir() if (p / "codelaw-pack.toml").is_file()):
        name = tomllib.loads((pack / "codelaw-pack.toml").read_text(encoding="utf-8"))["name"]
        to = work / "node_modules" / name if name.startswith("@") else work / "site-packages" / name.replace("-", "_")
        if to.exists():
            shutil.rmtree(to)
        shutil.copytree(pack, to)


# --------------------------------------------------------------- timing

class Series:
    def __init__(self, tool, op, mode, repo, lang=None, versus=None):
        self.tool, self.op, self.mode, self.repo, self.lang, self.versus = tool, op, mode, repo, lang, versus
        self.samples, self.exits = [], []

    def median(self):
        s, n = sorted(self.samples), len(self.samples)
        return s[n // 2] if n % 2 else (s[n // 2 - 1] + s[n // 2]) / 2

    def p95(self):
        s = sorted(self.samples)
        return s[min(max(-(-95 * len(s) // 100), 1), len(s)) - 1]  # nearest rank: ceil(0.95 n)


def fill(args, cache, tools):
    return [a.replace("{cache}", str(cache)).replace("{tools}", str(tools)) for a in args]


def once(cmd, cwd, env, ok_exit, logs, error_prefix=None):
    out, err = logs / "stdout", logs / "stderr"
    with out.open("wb") as o, err.open("wb") as e:
        started = time.perf_counter()
        code = subprocess.run(cmd, cwd=cwd, env=env, stdout=o, stderr=e, stdin=subprocess.DEVNULL).returncode
        ms = (time.perf_counter() - started) * 1000
    text = err.read_text(encoding="utf-8", errors="replace")
    if code not in ok_exit or (error_prefix and any(l.startswith(error_prefix) for l in text.splitlines())):
        tail = "\n".join(text.strip().splitlines()[-5:])
        raise RuntimeError(f"`{' '.join(cmd)}` in {cwd} exited {code}:\n{tail}")
    return ms, code


def measure(series, cmd, cwd, env, tool, runs, caches, logs, tools):
    """`runs` timed runs, the first discarded (codelaw bench's method)."""
    ok, prefix = tool["ok_exit"], tool.get("error_prefix")
    warm = caches / f"warm-{series.tool}-{series.op}-{series.repo}-{series.lang or 'all'}"
    if series.mode == "warm":
        warm.mkdir(parents=True, exist_ok=True)
        once(fill(cmd, warm, tools), cwd, fill_env(env, warm, tools, tool), ok, logs, prefix)  # prime
    for i in range(runs):
        cache = warm if series.mode == "warm" else caches / f"cold-{i}"
        if series.mode == "cold":
            shutil.rmtree(cache, ignore_errors=True)
            cache.mkdir(parents=True)
        ms, code = once(fill(cmd, cache, tools), cwd, fill_env(env, cache, tools, tool), ok, logs, prefix)
        if series.mode == "cold":
            shutil.rmtree(cache, ignore_errors=True)
        if i:
            series.samples.append(ms)
            series.exits.append(code)
    shutil.rmtree(warm, ignore_errors=True)
    return series


def fill_env(env, cache, tools, tool):
    e = dict(env)
    for k, v in tool.get("env", {}).items():
        e[k] = v.replace("{cache}", str(cache)).replace("{tools}", str(tools))
    return e


def run(table, args):
    if args.runs < 2:
        die("--runs is at least 2: the first run is discarded")
    only = set(args.only.split(",")) if args.only else None
    names = runnable(table, only)
    if "codelaw" not in names:
        die("codelaw is the reference: it runs in every bench (add it to --only)")
    binary = Path(args.binary).resolve()
    if not binary.is_file():
        die(f"--binary {args.binary}: no such file (cargo build --release -p codelaw-cli)")
    repos = args.repo or table["bench"]["repos"]
    out = Path(args.out).resolve()
    work, caches, logs = out / "work", out / "caches", out / "logs"
    for d in (work, caches, logs):
        shutil.rmtree(d, ignore_errors=True)
        d.mkdir(parents=True)
    install_packs(work)
    env = {k: v for k, v in os.environ.items() if k not in ("VIRTUAL_ENV", "CODELAW_SITE_PACKAGES", "CODELAW_CACHE_DIR")}
    env.update(NO_COLOR="1", CODELAW_SITE_PACKAGES=str(work / "site-packages"))
    strip = set(table["bench"]["strip"])
    tools_dir = Path(args.tools).resolve()
    rivals = {n: t for n, t in table["tool"].items() if n != "codelaw"}
    results, files = [], {}
    for repo in repos:
        src = Path(args.corpus).resolve() / "repos" / repo
        if not src.is_dir():
            die(f"{src}: no such folder (python3 scripts/corpus.py fetch --dir {args.corpus} --repo {repo})")
        for name in names:
            tool = table["tool"][name]
            cwd = work / name / repo
            files[repo] = copy_repo(src, cwd, strip)
            if "config" in tool:
                (cwd / tool["config_file"]).write_text(tool["config"].lstrip().replace("{tools}", str(tools_dir / name)), encoding="utf-8")
            bins = bin_dir(table, name, tools_dir, binary)
            modes = ("cold", "warm") if tool["cache"] else ("cold",)
            # codelaw in full, then scoped to each rival's languages
            scopes = [(None, None)] + ([(t["codelaw_lang"], r) for r, t in rivals.items()] if name == "codelaw" else [])
            for op in OPS:
                if op not in tool:
                    continue
                cmd = [a.replace("{bin}", str(bins)) for a in tool[op]]
                for lang, versus in scopes:
                    if versus and op not in rivals[versus]:
                        continue
                    for mode in modes:
                        s = Series(name, op, mode, repo, lang, versus)
                        c = cmd + (["--lang", lang] if lang else [])
                        print(f"  {repo:<28} {name:<8} {op:<6} {mode:<4} {('--lang ' + lang) if lang else ''}", flush=True)
                        try:
                            results.append(measure(s, c, cwd, env, tool, args.runs, caches, logs, tools_dir / name))
                        except RuntimeError as e:
                            die(str(e))
            shutil.rmtree(cwd, ignore_errors=True)
    report = {
        "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "commit": subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, capture_output=True, text=True).stdout.strip(),
        "machine": f"{platform.system()} {platform.machine()}, {os.cpu_count()} CPUs" + (f", {os.environ['ImageOS']} {os.environ.get('ImageVersion', '')}".rstrip() if "ImageOS" in os.environ else ""),
        "runs": args.runs,
        "dry_run": names == ["codelaw"],
        "tools": {n: {k: table["tool"][n][k] for k in ("version", "licence", "l11", "rule_set", "languages")} for n in names},
        "not_run": {n: l11_state(t)[1] for n, t in rivals.items() if n not in names},
        "repos": files,
        "rows": [{"repo": s.repo, "tool": s.tool, "op": s.op, "mode": s.mode, "lang": s.lang, "versus": s.versus,
                  "median_ms": round(s.median(), 1), "p95_ms": round(s.p95(), 1), "min_ms": round(min(s.samples), 1),
                  "max_ms": round(max(s.samples), 1), "exits": sorted(set(s.exits))} for s in results],
    }
    shutil.rmtree(work, ignore_errors=True)
    shutil.rmtree(caches, ignore_errors=True)
    (out / "results.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    md = markdown(report)
    (out / "results.md").write_text(md, encoding="utf-8")
    print(md)
    return 0


def markdown(r):
    head = "DRY RUN — codelaw alone; no rival is ratified (L11)" if r["dry_run"] else "side by side"
    s = [f"### bench-rivals: {head}", "",
         f"{r['date']} · commit `{r['commit']}` · {r['machine']} · {r['runs']} runs per row, first discarded; median and nearest-rank p95, spawn to exit", ""]
    s += ["| tool | version | rule set | languages |", "|---|---|---|---|"]
    s += [f"| {n} | {t['version']} | {t['rule_set']} | {t['languages']} |" for n, t in r["tools"].items()]
    if r["not_run"]:
        s += ["", "Not run: " + "; ".join(f"**{n}** — {why}" for n, why in r["not_run"].items())]
    for repo, n in r["repos"].items():
        s += ["", f"#### {repo} ({n} files)", "", "| tool | op | cache | scope | median ms | p95 ms | min – max ms | exit |", "|---|---|---|---|---|---|---|---|"]
        for row in (x for x in r["rows"] if x["repo"] == repo):
            scope = f"`--lang {row['lang']}` (vs {row['versus']})" if row["lang"] else "all its languages"
            s.append(f"| {row['tool']} | {row['op']} | {row['mode']} | {scope} | {row['median_ms']} | {row['p95_ms']} | {row['min_ms']} – {row['max_ms']} | {','.join(map(str, row['exits']))} |")
    s += ["", "Context rows (L8 §Measurements), never a budget. A tool with no cache is cold only. Compare a rival with the codelaw row scoped to its languages."]
    return "\n".join(s) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--self-test", action="store_true")
    sub = ap.add_subparsers(dest="cmd")
    sub.add_parser("plan")
    g = sub.add_parser("gate")
    g.add_argument("--github-output")
    i = sub.add_parser("install")
    i.add_argument("--tools", required=True)
    i.add_argument("--only")
    r = sub.add_parser("run")
    r.add_argument("--binary", required=True)
    r.add_argument("--corpus", default=str(ROOT / "target" / "corpus"), help="corpus.py fetch's --dir: repos are under <dir>/repos/")
    r.add_argument("--tools", default=str(ROOT / "target" / "bench-rivals" / "tools"))
    r.add_argument("--only", help="comma-separated tools; codelaw is always one")
    r.add_argument("--repo", action="append", default=[], help="a repo in [bench] repos (repeatable); all three by default")
    r.add_argument("--runs", type=int, default=None)
    r.add_argument("--out", default=str(ROOT / "target" / "bench-rivals"))
    a = ap.parse_args()
    table = load()
    if a.self_test:
        return self_test(table)
    if a.cmd == "plan":
        return plan(table)
    if a.cmd == "gate":
        return gate(table, a.github_output)
    if a.cmd == "install":
        return install(table, Path(a.tools).resolve(), set(a.only.split(",")) if a.only else None)
    if a.cmd == "run":
        a.runs = a.runs or table["bench"]["runs"]
        return run(table, a)
    ap.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
