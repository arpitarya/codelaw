#!/usr/bin/env python3
"""The pinned corpus: level 4 of `codelaw test` over real repositories
(SR-RULE-AUTHORING decision 4; W-20).

    corpus.py --self-test                    the manifest is well-formed (no network)
    corpus.py fetch [--dir D] [--pack P] [--repo NAME ...]
                                             check out each pinned repo, verify its licence
    corpus.py run --binary B [--dir D] [--pack P] [--update]
                                             per pack: codelaw test packs/P --corpus D/packs/P
                                             --snapshot packs/P/corpus.toml [--update-snapshot]

scripts/corpus.json names each repository, its full commit, its licence and
the packs that run over it. `fetch` clones each at exactly that commit (a
shallow fetch of the one commit) and refuses a repo whose licence file does
not carry its declared permissive licence (SR-LAW-2 decision 2): the corpus
is read, never copied into this repo or shipped, but it is held to the same
list. `run` assembles each pack's corpus from its repos and runs the pack's
rules over it; a finding count that moved, a missing count, or a broken
invariant fails. `--update` rewrites packs/<pack>/corpus.toml instead: review
that diff. The standard library only, Python 3.9.
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
PACKS = ("css", "data", "html", "js", "node", "python", "react", "stencil", "tanstack-query", "ts")  # FIRST_PARTY_PACKS since W-31; tanstack-query since W-42

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


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("action", nargs="?", choices=["fetch", "run"])
    ap.add_argument("--dir", type=Path, default=ROOT / "target" / "corpus")
    ap.add_argument("--pack", choices=PACKS)
    ap.add_argument("--repo", action="append", default=[], help="fetch only this repo (repeatable): `perf`'s three (L8 decision 2)")
    ap.add_argument("--binary", type=Path)
    ap.add_argument("--update", action="store_true", help="rewrite packs/<pack>/corpus.toml")
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
    ap.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
