#!/usr/bin/env python3
"""The pinned corpus: level 4 of `codelaw test` over real repositories
(SR-RULE-AUTHORING decision 4; W-20).

    corpus.py --self-test                    the manifest is well-formed (no network)
    corpus.py fetch [--dir D] [--pack P] [--repo NAME ...]
                                             check out each pinned repo, verify its licence
    corpus.py run --binary B [--dir D] [--pack P] [--update]
                                             per pack: codelaw test packs/P --corpus D/packs/P
                                             --snapshot packs/P/corpus.toml [--update-snapshot]
    corpus.py promote [--dir D] [--pack P] [--rules] [--write --run R]
                                             which preview rules the promotion criterion
                                             passes; --write flips them to `stable`

scripts/corpus.json names each repository, its full commit, its licence and
the packs that run over it. `fetch` clones each at exactly that commit (a
shallow fetch of the one commit) and refuses a repo whose licence file does
not carry its declared permissive licence (SR-LAW-2 decision 2): the corpus
is read, never copied into this repo or shipped, but it is held to the same
list. `run` assembles each pack's corpus from its repos and runs the pack's
rules over it; a finding count that moved, a missing count, or a broken
invariant fails. `--update` rewrites packs/<pack>/corpus.toml instead: review
that diff.

`promote` (W-100) reads, never runs: each rule's front matter, its count in
packs/<pack>/corpus.toml (which `run` has just held to the corpus), the
fetched repos, and corpus.json's `promotion` block — the criterion, the
reviewed findings and the promotions so far. Per rule it says whether the
criterion passes and, if not, why. `--write` sets `stability: stable` on each
rule that passes and records the run in `promotion.promoted`; it refuses while
`promotion.ratified` is null, so no rule is promoted on a criterion the
decider has not ratified (SR-RULE-AUTHORING decision 6a). The standard library
only, Python 3.9.
"""
import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

for _s in (sys.stdout, sys.stderr):
    if hasattr(_s, "reconfigure"):
        _s.reconfigure(encoding="utf-8")

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "scripts" / "corpus.json"
PACKS = ("css", "data", "html", "js", "node", "python", "react", "scss", "security-html", "security-js", "security-python", "security-ts", "stencil", "tanstack-query", "ts")  # FIRST_PARTY_PACKS since W-31; tanstack-query since W-42; the security packs since W-53; scss since W-62 (css-modules waits on W-63)

# Each allowed licence (a subset of SR-LAW-2 decision 2's list) and phrases its
# licence text must carry, all of them, whitespace-normalised.
LICENCES = {
    "MIT": ["Permission is hereby granted, free of charge", "THE SOFTWARE IS PROVIDED \"AS IS\""],
    "BSD-3-Clause": ["Redistribution and use in source and binary forms", "Neither the name"],
    "BSD-2-Clause": ["Redistribution and use in source and binary forms"],
    "Apache-2.0": ["Apache License", "Version 2.0"],
    "ISC": ["Permission to use, copy, modify, and/or distribute this software"],
}
SHA = re.compile(r"^[0-9a-f]{40}$")
NAME = re.compile(r"^[A-Za-z0-9._-]+$")


def fail(msg):
    print(f"✖ corpus: {msg}")
    sys.exit(1)


def dump(manifest):
    """corpus.json as it is written: two-space JSON, a list of strings on one line."""
    text = json.dumps(manifest, indent=2, ensure_ascii=False) + "\n"
    return re.sub(r'\[\n\s+("[^"\n]*"(?:,\n\s+"[^"\n]*")*)\n\s+\]', lambda m: "[" + re.sub(r",\n\s+", ", ", m.group(1)) + "]", text)


def load():
    return json.loads(MANIFEST.read_text(encoding="utf-8"))["repos"]


def problems(repos):
    """What is wrong with the manifest; empty when it is well-formed."""
    out, names = [], set()
    for r in repos:
        n = r.get("name", "?")
        if not NAME.match(n) or n in names:
            out.append(f"{n}: a name is unique and path-safe")
        names.add(n)
        if not str(r.get("url", "")).startswith("https://"):
            out.append(f"{n}: the url is https")
        if not SHA.match(str(r.get("commit", ""))):
            out.append(f"{n}: `commit` is a full 40-hex sha, not a branch or tag")
        if r.get("licence") not in LICENCES:
            out.append(f"{n}: licence {r.get('licence')!r} is not one SR-LAW-2 lets this script accept ({', '.join(LICENCES)})")
        if not r.get("licence_file"):
            out.append(f"{n}: `licence_file` names the file that carries the licence")
        packs = r.get("packs") or []
        if not packs or any(p not in PACKS for p in packs):
            out.append(f"{n}: `packs` lists one or more of {', '.join(PACKS)}")
    for p in PACKS:
        if not any(p in (r.get("packs") or []) for r in repos):
            out.append(f"pack {p}: no repo in the corpus")
    return out


def self_test():
    repos = load()
    bad = problems(repos)
    if bad:
        fail("scripts/corpus.json:\n  " + "\n  ".join(bad))
    # The checks catch what they claim to.
    broken = [dict(repos[0], commit="main", licence="GPL-3.0", packs=["nope"]), dict(repos[0])]
    found = "\n".join(problems(broken))
    for want in ("full 40-hex sha", "GPL-3.0", "`packs` lists", "unique", "no repo in the corpus"):
        if want not in found:
            fail(f"self-test: the manifest check misses {want!r}:\n{found}")
    if not licence_ok("MIT", "Permission is hereby granted, free of charge, to any\nperson ... THE SOFTWARE IS PROVIDED \"AS IS\""):
        fail("self-test: a wrapped MIT text is not recognised")
    if licence_ok("MIT", "GNU GENERAL PUBLIC LICENSE Version 3"):
        fail("self-test: a GPL text passed as MIT")
    text = MANIFEST.read_text(encoding="utf-8")
    if dump(json.loads(text)) != text:
        fail("self-test: scripts/corpus.json is not as `promote --write` would write it — two-space JSON, string lists on one line")
    promo = json.loads(text).get("promotion") or {}
    bad = criterion_problems(promo)
    if bad:
        fail("scripts/corpus.json:\n  " + "\n  ".join(bad))
    if criterion_problems(dict(promo, ratified="2026-11-02 Arpit: yes")) or not criterion_problems(dict(promo, ratified="yes")):
        fail("self-test: `promotion.ratified` is not held to a dated ruling")
    for g, path, want in (("**/package.json", "package.json", True), ("**/package.json", "a/b/package.json", True),
                          ("src/**", "lib/x.py", False), ("**/test_*.py", "t/test_a.py", True), ("**/*.{yml,yaml}", "a.yaml", True)):
        if bool(glob_re(g).match(path)) != want:
            fail(f"self-test: glob {g!r} on {path!r} should be {want}")
    crit = {"criterion": {"min_repos": 2, "min_files": 2, "zero_findings_is_evidence": True, "min_releases_in_preview": 1}}
    two = {"a": ["x.css"], "b": ["y.scss", "z.py"]}
    rule = {"id": "css/x", "languages": "[css, scss]", "since": "0.2.0"}
    cases = (
        (rule, 0, two, crit, True, ""),
        (rule, 0, {"a": ["x.css", "y.css"]}, crit, False, "1 corpus repos"),
        (rule, 3, two, crit, False, "none reviewed"),
        (rule, 3, two, dict(crit, reviewed={"css/x": {"findings": 2, "false_positives": 0}}), False, "now gives 3"),
        (rule, 3, two, dict(crit, reviewed={"css/x": {"findings": 3, "false_positives": 1}}), False, "1 false positives"),
        (rule, 3, two, dict(crit, reviewed={"css/x": {"findings": 3, "false_positives": 0}}), True, ""),
        (dict(rule, since="0.3.0"), 0, two, crit, False, "published release"),
        (rule, None, two, crit, False, "no count"),
        (dict(rule, files='["**/package.json"]'), 0, two, crit, False, "0 corpus files"),
    )
    for r, n, rp, pr, want, why in cases:
        ok, reasons = verdict(r, n, rp, pr, (0, 2, 0))
        if ok != want or why not in "; ".join(w for _, w in reasons):
            fail(f"self-test: verdict({r.get('since')}, {n}) = {ok}, {reasons}; want {want}, {why!r}")
    print(f"✔ corpus manifest: {len(repos)} repos, every pack covered, every commit pinned, every licence permissive")


def licence_ok(spdx, text):
    flat = " ".join(text.split())
    return all(" ".join(p.split()) in flat for p in LICENCES[spdx])


def git(*args, cwd=None):
    return subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


def selected(repos, pack):
    return [r for r in repos if pack is None or pack in r["packs"]]


def fetch(repos, base):
    for r in repos:
        d = base / "repos" / r["name"]
        if not (d / ".git").is_dir() or git("rev-parse", "HEAD", cwd=d) != r["commit"]:
            shutil.rmtree(d, ignore_errors=True)
            d.mkdir(parents=True)
            git("init", "-q", cwd=d)
            git("fetch", "-q", "--depth", "1", r["url"], r["commit"], cwd=d)
            git("-c", "advice.detachedHead=false", "checkout", "-q", "--detach", "FETCH_HEAD", cwd=d)
        head = git("rev-parse", "HEAD", cwd=d)
        if head != r["commit"]:
            fail(f"{r['name']}: checked out {head}, the manifest pins {r['commit']}")
        lic = d / r["licence_file"]
        if not lic.is_file() or not licence_ok(r["licence"], lic.read_text(encoding="utf-8", errors="replace")):
            fail(f"{r['name']}: {r['licence_file']} at {r['commit'][:12]} is not {r['licence']} — the corpus holds only permissive code (SR-LAW-2)")
        print(f"✔ {r['name']} @ {r['commit'][:12]} · {r['licence']}")


def run(repos, base, binary, pack, update):
    code = 0
    for p in PACKS if pack is None else (pack,):
        mine = selected(repos, p)
        corpus = base / "packs" / p
        shutil.rmtree(corpus, ignore_errors=True)
        for r in mine:
            src = base / "repos" / r["name"]
            if not (src / ".git").is_dir():
                fail(f"{r['name']}: not fetched — run `corpus.py fetch` first")
            shutil.copytree(src, corpus / r["name"], ignore=shutil.ignore_patterns(".git"), symlinks=True)
        cmd = [str(binary), "test", f"packs/{p}", "--corpus", str(corpus), "--snapshot", f"packs/{p}/corpus.toml"]
        if update:
            cmd.append("--update-snapshot")
        print(f"\n── {p}: {', '.join(r['name'] for r in mine)}", flush=True)
        res = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, encoding="utf-8")
        failed = [l for l in res.stdout.splitlines() if l.startswith("✖") or l.startswith("    ")]
        summary = [l for l in res.stdout.splitlines() if l.startswith("corpus:") or " rules · " in l]
        print("\n".join(failed + summary))
        if res.stderr.strip():
            print(res.stderr.strip())
        if res.returncode != 0:
            code = 1
    return code


# The L3 names a rule's `languages:` uses, and the extensions the engine reads
# as each (codelaw-core's Language::from_path).
EXTENSIONS = {
    "javascript": ("js", "mjs", "cjs"), "jsx": ("jsx",), "typescript": ("ts", "mts", "cts"), "tsx": ("tsx",),
    "python": ("py", "pyi"), "json": ("json", "jsonc"), "yaml": ("yml", "yaml"), "toml": ("toml",),
    "markdown": ("md", "mdx"), "css": ("css",), "scss": ("scss",), "html": ("html", "htm"),
}
SKIP_DIRS = {".git", "node_modules", ".venv", ".codelaw"}
CRITERION_KEYS = {"min_repos": int, "min_files": int, "zero_findings_is_evidence": bool, "min_releases_in_preview": int}


def front_matter(text):
    """The rule file's top-level front-matter fields, as raw strings."""
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---", 3)
    out = {}
    for line in text[4:end].splitlines():
        m = re.match(r"^([a-z_]+):\s*(.*)$", line)
        if m:
            out[m.group(1)] = m.group(2).strip()
    return out


def flow_list(v):
    """`[a, "b/**"]` → ['a', 'b/**']."""
    v = (v or "").strip()
    if not v.startswith("["):
        return [v.strip("\"'")] if v else []
    return [x.strip().strip("\"'") for x in v[1:-1].split(",") if x.strip()]


def glob_re(g):
    """A rule's `files:`/`exclude:` glob as a regex over a repo-relative path."""
    out, i = "", 0
    while i < len(g):
        c = g[i]
        if g.startswith("**/", i):
            out, i = out + "(?:.*/)?", i + 3
        elif g.startswith("**", i):
            out, i = out + ".*", i + 2
        elif c == "*":
            out, i = out + "[^/]*", i + 1
        elif c == "?":
            out, i = out + "[^/]", i + 1
        elif c == "{":
            j = g.index("}", i)
            out, i = out + "(?:" + "|".join(re.escape(x) for x in g[i + 1:j].split(",")) + ")", j + 1
        else:
            out, i = out + re.escape(c), i + 1
    return re.compile("^" + out + "$")


def repo_paths(d):
    """Every file under a fetched repo, repo-relative, in path order."""
    out = []
    for p in sorted(d.rglob("*")):
        rel = p.relative_to(d)
        if any(part in SKIP_DIRS for part in rel.parts[:-1]) or not p.is_file():
            continue
        out.append(rel.as_posix())
    return out


def version(v):
    return tuple(int(x) for x in re.findall(r"\d+", v)[:3])


def criterion_problems(promo):
    out = []
    crit = promo.get("criterion") or {}
    for k, t in CRITERION_KEYS.items():
        if not isinstance(crit.get(k), t) or (t is int and isinstance(crit.get(k), bool)):
            out.append(f"promotion.criterion.{k}: a {t.__name__}")
    for k in set(crit) - set(CRITERION_KEYS):
        out.append(f"promotion.criterion.{k}: not a criterion key ({', '.join(CRITERION_KEYS)})")
    r = promo.get("ratified")
    if r is not None and not (isinstance(r, str) and re.match(r"^\d{4}-\d{2}-\d{2} ", r)):
        out.append("promotion.ratified: null, or the decider's date and words (`2026-11-02 Arpit: …`)")
    for rid, rev in (promo.get("reviewed") or {}).items():
        if not all(isinstance(rev.get(k), int) for k in ("findings", "false_positives")) or not rev.get("by") or not rev.get("on"):
            out.append(f"promotion.reviewed.{rid}: findings, false_positives, by, on")
    return out


def verdict(rule, count, repos, promo, current):
    """(passes, [(kind, reason)] it does not) for one preview rule — the criterion of
    corpus.json's `promotion` block (proposed in SR-RULE-AUTHORING, W-100)."""
    crit = promo["criterion"]
    why = []
    if count is None:
        why.append(("no count in corpus.toml", "no count in corpus.toml"))
    langs = flow_list(rule.get("languages"))
    exts = tuple("." + e for l in langs for e in EXTENSIONS.get(l, ()))
    inc = [glob_re(g) for g in flow_list(rule.get("files"))]
    exc = [glob_re(g) for g in flow_list(rule.get("exclude"))]
    hit_repos, files = 0, 0
    for paths in repos.values():
        n = sum(1 for p in paths if p.lower().endswith(exts) and (not inc or any(r.match(p) for r in inc)) and not any(r.match(p) for r in exc))
        hit_repos += n > 0
        files += n
    if hit_repos < crit["min_repos"]:
        why.append((f"fewer than {crit['min_repos']} corpus repos hold its files", f"{hit_repos} corpus repos hold its files (needs {crit['min_repos']})"))
    if files < crit["min_files"]:
        why.append((f"fewer than {crit['min_files']} corpus files", f"{files} corpus files (needs {crit['min_files']})"))
    rev = (promo.get("reviewed") or {}).get(rule["id"])
    if count:
        if rev is None:
            why.append(("findings not reviewed", f"{count} findings, none reviewed"))
        elif rev["findings"] != count:
            why.append(("review out of date", f"reviewed at {rev['findings']} findings, the corpus now gives {count}"))
        elif rev["false_positives"]:
            why.append(("false positives", f"{rev['false_positives']} false positives of {count}"))
    elif count == 0 and not crit["zero_findings_is_evidence"]:
        why.append(("0 findings: nothing to review", "0 findings: nothing to review"))
    # Cargo.toml's version is the last release; a rule since it has shipped in one.
    if version(rule.get("since") or "0.0.0") > (current[0], current[1] - crit["min_releases_in_preview"] + 1, 10**6):
        why.append(("not yet in a published release", f"since {rule.get('since')}: not yet in {crit['min_releases_in_preview']} published release(s) as preview"))
    return not why, why


def promote(repos, base, pack, show_rules, write, run_ref):
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    promo = manifest.get("promotion") or {}
    bad = criterion_problems(promo)
    if bad:
        fail("scripts/corpus.json:\n  " + "\n  ".join(bad))
    if write and not promo.get("ratified"):
        fail("--write: the promotion criterion is a proposal — `promotion.ratified` is null until the decider rules (SR-RULE-AUTHORING decision 6a)")
    if write and not run_ref:
        fail("--write needs --run <the corpus run that proved it: a CI run URL>")
    m = re.search(r'^version = "([^"]+)"', (ROOT / "Cargo.toml").read_text(encoding="utf-8"), re.M)
    current = version(m.group(1))
    state = "ratified " + promo["ratified"] if promo.get("ratified") else "PROPOSED, not ratified — nothing is promoted on it"
    print(f"promotion criterion ({state}): {json.dumps(promo['criterion'], sort_keys=True)}")
    total, passing = 0, []
    for p in PACKS if pack is None else (pack,):
        paths = {}
        for r in selected(repos, p):
            d = base / "repos" / r["name"]
            if not (d / ".git").is_dir():
                fail(f"{r['name']}: not fetched — run `corpus.py fetch` first")
            paths[r["name"]] = repo_paths(d)
        snap = ROOT / "packs" / p / "corpus.toml"
        counts = {k: int(v) for k, v in re.findall(r'^"([^"]+)" = (\d+)$', snap.read_text(encoding="utf-8") if snap.is_file() else "", re.M)}
        rules, held = [], {}
        for f in sorted((ROOT / "packs" / p / "rules").rglob("*.md")):
            if f.name.endswith(".cases.md"):
                continue
            rule = front_matter(f.read_text(encoding="utf-8"))
            if not rule.get("id") or rule.get("stability", "preview") != "preview":
                continue
            ok, why = verdict(rule, counts.get(rule["id"]), paths, promo, current)
            rules.append((rule["id"], f, ok, why))
            for kind, _ in why:
                held[kind] = held.get(kind, 0) + 1
        n_ok = sum(1 for r in rules if r[2])
        total += len(rules)
        passing += [(rid, f) for rid, f, ok, _ in rules if ok]
        print(f"\n── {p}: {len(rules)} preview rules · {n_ok} pass · {len(rules) - n_ok} held ({', '.join(paths)})")
        for w, n in sorted(held.items(), key=lambda x: (-x[1], x[0])):
            print(f"    {n:4}  {w}")
        if show_rules:
            for rid, _, ok, why in rules:
                print(f"  {'✔' if ok else '·'} {rid}" + ("" if ok else "  — " + "; ".join(w for _, w in why)))
    print(f"\npromotion: {len(passing)} of {total} preview rules pass")
    if write:
        import datetime
        today = datetime.date.today().isoformat()
        for rid, f in passing:
            text = f.read_text(encoding="utf-8")
            new = re.sub(r"^stability: preview$", "stability: stable", text, count=1, flags=re.M)
            if new == text:
                new = re.sub(r"^(fix: .*)$", r"\1\nstability: stable", text, count=1, flags=re.M)
            f.write_text(new, encoding="utf-8")
            promo.setdefault("promoted", {})[rid] = {"on": today, "run": run_ref}
        MANIFEST.write_text(dump(manifest), encoding="utf-8")
        print(f"wrote stability: stable on {len(passing)} rules; each is in promotion.promoted with {run_ref}")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("action", nargs="?", choices=["fetch", "run", "promote"])
    ap.add_argument("--dir", type=Path, default=ROOT / "target" / "corpus")
    ap.add_argument("--pack", choices=PACKS)
    ap.add_argument("--repo", action="append", default=[], help="fetch only this repo (repeatable): `perf`'s three (L8 decision 2)")
    ap.add_argument("--binary", type=Path)
    ap.add_argument("--update", action="store_true", help="rewrite packs/<pack>/corpus.toml")
    ap.add_argument("--rules", action="store_true", help="promote: one line per rule, with why it is held")
    ap.add_argument("--write", action="store_true", help="promote: set `stability: stable` on each rule that passes (needs a ratified criterion)")
    ap.add_argument("--run", help="promote --write: the corpus run that proved them (a CI run URL)")
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return 0
    repos = load()
    bad = problems(repos)
    if bad:
        fail("scripts/corpus.json:\n  " + "\n  ".join(bad))
    base = a.dir.resolve()
    if a.action == "fetch":
        unknown = set(a.repo) - {r["name"] for r in repos}
        if unknown:
            fail(f"--repo {', '.join(sorted(unknown))}: not in scripts/corpus.json")
        fetch([r for r in selected(repos, a.pack) if not a.repo or r["name"] in a.repo], base)
        return 0
    if a.action == "run":
        if not a.binary or not a.binary.is_file():
            fail("run needs --binary <the codelaw binary>")
        return run(repos, base, a.binary.resolve(), a.pack, a.update)
    if a.action == "promote":
        return promote(repos, base, a.pack, a.rules, a.write, a.run)
    ap.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
