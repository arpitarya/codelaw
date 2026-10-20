#!/usr/bin/env python3
"""format-parity — how close `codelaw format` is to Prettier and Black.

SR-FORMATTER promises Prettier-compatible JS/TS/JSON/CSS/HTML/YAML/Markdown
and Black-compatible Python, "compatible" not "identical". This script
measures the gap on a pinned corpus and prints one table, which goes into
SR-FORMATTER's Consequences.

    python3 scripts/format-parity.py --binary target/release/codelaw

The corpus is pinned by the commit it runs on:
  - every fenced example in `packs/**/*.md` (the rules' Do and Don't code),
    one file per block;
  - every tracked file of an L3 language outside `packs/`, `archive/` and
    `crates/*/tests/`.
Prettier and Black are run at the pinned versions below through `npx` and
`uvx`; they are measuring instruments only, never a codelaw dependency (L2).
Each tool formats its own copy; a file either tool cannot format (a
fragment, a deliberate syntax error) is counted as skipped, not compared.
TOML has no Prettier or Black formatter and is left out.
"""
import argparse
import difflib
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

PRETTIER = "prettier@3.6.2"
BLACK = "black==25.9.0"

# fence tag -> (extension, reference tool)
FENCES = {
    "js": ("js", "prettier"), "jsx": ("jsx", "prettier"), "ts": ("ts", "prettier"), "tsx": ("tsx", "prettier"),
    "json": ("json", "prettier"), "css": ("css", "prettier"), "html": ("html", "prettier"),
    "yaml": ("yaml", "prettier"), "py": ("py", "black"),
}
EXTENSIONS = {"js", "jsx", "mjs", "cjs", "ts", "tsx", "json", "css", "html", "yaml", "yml", "md", "py"}
LANGUAGE = {"js": "JS · JSX", "jsx": "JS · JSX", "mjs": "JS · JSX", "cjs": "JS · JSX", "ts": "TS · TSX", "tsx": "TS · TSX",
            "json": "JSON", "css": "CSS", "html": "HTML", "yaml": "YAML", "yml": "YAML", "md": "Markdown", "py": "Python"}
FENCE = re.compile(r"^```([a-z]+)\n(.*?)^```", re.M | re.S)


def corpus(repo: pathlib.Path, out: pathlib.Path) -> None:
    n = 0
    for md in sorted((repo / "packs").rglob("*.md")):
        for tag, body in FENCE.findall(md.read_text(encoding="utf-8")):
            if tag in FENCES:
                n += 1
                (out / f"pack-{n:05d}.{FENCES[tag][0]}").write_text(body, encoding="utf-8")
    tracked = subprocess.run(["git", "ls-files"], cwd=repo, capture_output=True, text=True, check=True).stdout.split()
    for rel in tracked:
        ext = rel.rsplit(".", 1)[-1]
        if ext not in EXTENSIONS or rel.startswith(("packs/", "archive/")) or "/tests/" in rel:
            continue
        dest = out / ("repo-" + rel.replace("/", "__"))
        shutil.copyfile(repo / rel, dest)


def run(cmd, cwd):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--binary", required=True, help="the codelaw binary to measure")
    args = ap.parse_args()
    repo = pathlib.Path(__file__).resolve().parent.parent
    binary = pathlib.Path(args.binary).resolve()
    commit = run(["git", "rev-parse", "--short", "HEAD"], repo).stdout.strip()
    with tempfile.TemporaryDirectory() as tmp:
        base = pathlib.Path(tmp)
        src = base / "src"
        src.mkdir()
        corpus(repo, src)
        files = sorted(p.name for p in src.iterdir())
        outs = {}
        for tool in ("codelaw", "prettier", "black"):
            d = base / tool
            shutil.copytree(src, d)
            outs[tool] = d
        (outs["codelaw"] / "codelaw.toml").write_text("", encoding="utf-8")
        # Files a tool could not format, from its own report; they are
        # skipped, not compared.
        failed = set()
        r = run([str(binary), "format", "--no-cache", "."], outs["codelaw"])
        current = None
        for line in r.stdout.splitlines():
            if line and not line[0].isspace():
                current = line.strip()
            elif "format/failed" in line and current:
                failed.add(current)
        # Prettier skips what it has no parser for (`.py`); a file it
        # cannot parse is reported on stderr and left as it was.
        r = run(["npx", "--yes", PRETTIER, "--write", "--no-config", "--log-level", "warn", "."], outs["prettier"])
        if r.returncode not in (0, 2):
            sys.exit(f"format-parity: prettier failed:\n{r.stderr}")
        failed.update(m.group(1) for m in re.finditer(r"^\[error\] (?:\./)?([^:\s]+):", r.stderr, re.M))
        r = run(["uvx", BLACK, "--quiet", "."], outs["black"])
        if r.returncode not in (0, 123):
            sys.exit(f"format-parity: black failed:\n{r.stderr}")
        failed.update(m.group(1) for m in re.finditer(r"^error: cannot format (?:\./)?([^:\s]+):", r.stderr, re.M))

        rows = {}
        for f in files:
            ext = f.rsplit(".", 1)[-1]
            ref = "black" if ext == "py" else "prettier"
            ours = (outs["codelaw"] / f).read_text(encoding="utf-8")
            theirs = (outs[ref] / f).read_text(encoding="utf-8")
            row = rows.setdefault((LANGUAGE[ext], ref), {"files": 0, "same": 0, "lines": 0, "same_lines": 0, "skipped": 0})
            if f in failed:
                row["skipped"] += 1
                continue
            row["files"] += 1
            row["same"] += ours == theirs
            a, b = ours.splitlines(), theirs.splitlines()
            matcher = difflib.SequenceMatcher(None, a, b, autojunk=False)
            row["lines"] += max(len(a), len(b))
            row["same_lines"] += sum(m.size for m in matcher.get_matching_blocks())
    print(f"Corpus: commit {commit}, {len(files)} files; {PRETTIER}, {BLACK}.\n")
    print("| language | vs | files compared | identical files | identical lines | skipped |")
    print("|---|---|---|---|---|---|")
    for (lang, ref), r in sorted(rows.items()):
        pf = 100.0 * r["same"] / r["files"] if r["files"] else 0.0
        pl = 100.0 * r["same_lines"] / r["lines"] if r["lines"] else 0.0
        print(f"| {lang} | {ref.capitalize()} | {r['files']} | {pf:.1f} % | {pl:.1f} % | {r['skipped']} |")
    return 0


if __name__ == "__main__":
    sys.exit(main())
