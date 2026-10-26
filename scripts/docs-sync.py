#!/usr/bin/env python3
"""docs-sync — the public docs' numbers and tables, from the tree (W-26).

SR-WORK-REGISTRY's Consequences: `README.md`'s and `CHANGELOG.md`'s update
triggers are checked here, not remembered. What it reads, and what it holds
the docs to:

  counts     every rule file (`packs/*/rules/**/*.md`, not `*.cases.md`): its
             pack, its `fix:` class, whether its block is `native:`; every
             preset's `[rules]`; the registries from FIRST_PARTY_PACKS
             (through `scripts/package.py`). They fill README.md's pack and
             fix-class tables, the counts in its prose, USAGE.md's install loops, `packaging/README.md`'s
             registry table and each pack README's preset table.
  commands   the subcommands of `codelaw --help` — from the binary with
             `--codelaw BIN`, otherwise from the `Command` enum in
             `crates/codelaw-cli/src/main.rs` (clap's names). Every one has a
             row in USAGE.md's command table, and no row names one that is
             gone. `--write` adds a missing row from the help line.
  checks     with `--codelaw BIN` only: `codelaw test packs/<pack>` for every
             pack, summed into README.md's "N example checks".
  roadmap    each row of README.md's roadmap names a record; no row is
             "designed" or 🔜 for a record the register marks `built: yes`.
  changelog  every `W-nn` in a `work/IMPLEMENTATION.md` row dated after the
             last dated release in CHANGELOG.md is named under *Unreleased*;
             one dated on the release day is named there or in the release.
  api        every function the npm API (`index.d.ts`) and the Python module
             (`__all__`) export is named in `packaging/README.md`.

A generated region sits between `<!-- docs-sync:NAME -->` and
`<!-- /docs-sync:NAME -->`. In a table region only the computed cells are
rewritten; a prose cell (what a pack was ported from, what a fix class does)
is kept. A number in prose is an anchor: a pattern that must match exactly
once, so a reworded sentence fails the check instead of going stale.

    python3 scripts/docs-sync.py --check                      # CI (lint): exit 1 on drift
    python3 scripts/docs-sync.py --write                      # rewrite what can be computed
    python3 scripts/docs-sync.py --check --codelaw target/debug/codelaw

Stdlib only, Python 3.9 syntax (work/MACHINE.md).
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import package  # noqa: E402  FIRST_PARTY_PACKS, read once for the whole repo

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")  # work/MACHINE.md: the Windows console

README = ROOT / "README.md"
USAGE = ROOT / "USAGE.md"
CHANGELOG = ROOT / "CHANGELOG.md"
PKG_README = ROOT / "packaging" / "README.md"
IMPLEMENTATION = ROOT / "work" / "IMPLEMENTATION.md"
REGISTER = ROOT / "records" / "README.md"
MAIN_RS = ROOT / "crates" / "codelaw-cli" / "src" / "main.rs"
NPM_API = ROOT / "packaging" / "npm" / "codelaw" / "index.d.ts"
PY_API = ROOT / "packaging" / "python" / "python" / "codelaw" / "__init__.py"
FIX_CLASSES = ("auto", "suggest", "manual")
WORDS = ("zero", "one", "two", "three", "four", "five", "six", "seven", "eight",
         "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen",
         "sixteen", "seventeen", "eighteen", "nineteen", "twenty")


def word(n, capital=False):
    w = WORDS[n] if n < len(WORDS) else str(n)
    return w.capitalize() if capital else w


# ── what the tree says ──────────────────────────────────────────────────────

def frontmatter(text):
    m = re.match(r"---\r?\n(.*?)\r?\n---\r?\n", text, re.S)
    return m.group(1) if m else None


def read_packs():
    """{pack: {"rules": [(id, fix, native)], "presets": [(name, count, description)]}}

    A preset's count is every rule it turns on: its own `[rules]` and those
    of the first-party presets its `extends` names (SR-RULE-PACKS decision 3).
    """
    packs = {}
    own, extends = {}, {}
    for name in package.PACKS:
        pdir = ROOT / "packs" / name
        rules = []
        for f in sorted((pdir / "rules").rglob("*.md")):
            if f.name.endswith(".cases.md"):
                continue
            text = f.read_text(encoding="utf-8")
            fm = frontmatter(text)
            if fm is None:
                sys.exit("docs-sync: {} has no front matter".format(f.relative_to(ROOT)))
            rid = re.search(r"^id:\s*(\S+)", fm, re.M)
            fix = re.search(r"^fix:\s*(\S+)", fm, re.M)
            if not rid or not fix or fix.group(1) not in FIX_CLASSES:
                sys.exit("docs-sync: {} lacks `id:` or a `fix:` class".format(f.relative_to(ROOT)))
            block = re.search(r"```codelaw\r?\n(.*?)```", text, re.S)
            native = bool(block and re.search(r"^native:", block.group(1), re.M))
            rules.append((rid.group(1), fix.group(1), native))
        manifest = (pdir / "codelaw-pack.toml").read_text(encoding="utf-8")
        listed = re.search(r"^presets\s*=\s*\[(.*?)\]", manifest, re.M | re.S)
        presets = []
        for preset in re.findall(r'"([^"]+)"', listed.group(1) if listed else ""):
            ptext = (pdir / "presets" / (preset + ".toml")).read_text(encoding="utf-8")
            desc = re.search(r'^description\s*=\s*"(.*)"', ptext, re.M)
            head, body = ptext.split("[rules]", 1) if "[rules]" in ptext else (ptext, "")
            own[preset] = set(re.findall(r'^\s*"([^"]+)"\s*=', body, re.M))
            ext = re.search(r"^extends\s*=\s*\[(.*?)\]", head, re.M | re.S)
            extends[preset] = re.findall(r'"codelaw:([^"]+)"', ext.group(1)) if ext else []
            presets.append((preset, 0, desc.group(1) if desc else ""))
        packs[name] = {"rules": rules, "presets": presets}

    def turned_on(preset, seen=()):
        if preset in seen or preset not in own:
            sys.exit("docs-sync: preset `{}` extends a preset no pack has, or a cycle".format(preset))
        out = set(own[preset])
        for e in extends[preset]:
            out |= turned_on(e, seen + (preset,))
        return out

    for pack in packs.values():
        pack["presets"] = [(p, len(turned_on(p)), d) for p, _, d in pack["presets"]]
    return packs


def subcommands_from_source():
    """[(name, help line)] from main.rs's `enum Command`, as clap names them."""
    text = MAIN_RS.read_text(encoding="utf-8")
    body = text[text.index("enum Command {"):]
    out, doc, depth = [], [], 0
    for line in body.splitlines()[1:]:
        s = line.strip()
        if depth == 0 and s == "}":
            break
        if depth == 0 and s.startswith("///"):
            doc.append(s[3:].strip())
            continue
        m = re.match(r"    ([A-Z][A-Za-z0-9]*)\b", line) if depth == 0 else None
        if m:
            name = re.sub(r"(?<!^)([A-Z])", r"-\1", m.group(1)).lower()
            out.append((name, " ".join(doc).split(". ")[0].rstrip(".")))
            doc = []
        elif depth == 0 and not s.startswith("#["):
            doc = []
        depth += line.count("{") - line.count("}")
    return out


def subcommands_from_binary(binary):
    text = subprocess.run([binary, "--help"], capture_output=True, text=True, check=True).stdout
    block = text.split("Commands:", 1)[1].split("\n\n", 1)[0]
    return [(m.group(1), m.group(2).strip()) for m in re.finditer(r"^  ([a-z][a-z0-9-]*)\s+(.*)$", block, re.M)]


def example_checks(binary):
    total = 0
    for name in package.PACKS:
        run = subprocess.run([binary, "test", str(ROOT / "packs" / name)], capture_output=True, text=True)
        m = re.search(r"(\d+) rules · (\d+) checks · (\d+) failed", run.stdout + run.stderr)
        if not m or run.returncode != 0 or m.group(3) != "0":
            sys.exit("docs-sync: `codelaw test packs/{}` did not pass".format(name))
        total += int(m.group(2))
    return total


def register_built():
    out = {}
    for m in re.finditer(r"^\| \[\d{4}\]\([^)]*\) \| \*\*(SR-[A-Z0-9-]+)\*\* \|.*\| (\w+) \|\s*$",
                         REGISTER.read_text(encoding="utf-8"), re.M):
        out[m.group(1)] = m.group(2)
    return out


def api_names():
    npm = re.findall(r"^export function (\w+)", NPM_API.read_text(encoding="utf-8"), re.M)
    alls = re.search(r"^__all__\s*=\s*\[(.*?)\]", PY_API.read_text(encoding="utf-8"), re.M | re.S)
    py = re.findall(r'"(\w+)"', alls.group(1)) if alls else []
    return npm + py


# ── editing a document ──────────────────────────────────────────────────────

class Doc:
    def __init__(self, path):
        self.path = path
        self.text = path.read_text(encoding="utf-8")
        self.original = self.text
        self.problems = []

    def problem(self, msg):
        self.problems.append("{}: {}".format(self.path.relative_to(ROOT), msg))

    def anchor(self, pattern, values, optional=False):
        """Every group of the one match of `pattern` becomes `values[i]`."""
        matches = list(re.finditer(pattern, self.text))
        if len(matches) != 1:
            if matches or not optional:
                self.problem("the pattern {!r} matches {} times, not once".format(pattern, len(matches)))
            return
        m = matches[0]
        parts, last = [], m.start()
        for i, v in enumerate(values, 1):
            parts += [self.text[last:m.start(i)], str(v)]
            last = m.end(i)
        self.text = self.text[:m.start()] + "".join(parts) + self.text[last:]

    def region(self, name):
        begin, end = "<!-- docs-sync:{} -->".format(name), "<!-- /docs-sync:{} -->".format(name)
        if self.text.count(begin) != 1 or self.text.count(end) != 1:
            self.problem("needs exactly one {} … {}".format(begin, end))
            return None
        i = self.text.index(begin) + len(begin)
        return i, self.text.index(end, i)

    def table(self, name):
        """(rows as cell lists, the region's span) for the table in a region."""
        span = self.region(name)
        if span is None:
            return None, None
        rows = [split_row(l) for l in self.text[span[0]:span[1]].splitlines() if l.startswith("|")]
        return rows, span

    def put_table(self, span, rows):
        self.text = self.text[:span[0]] + "\n\n" + "\n".join(join_row(r) for r in rows) + "\n\n" + self.text[span[1]:]


def split_row(line):
    cells = re.split(r"(?<!\\)\|", line.strip())[1:-1]
    return [c.strip() for c in cells]


def join_row(cells):
    if cells and all(re.fullmatch(r":?-+:?", c) for c in cells):
        return "|" + "|".join(cells) + "|"
    return "| " + " | ".join(cells) + " |"


def sync_table(doc, name, keys, computed, new_row, key_of=None):
    """Rewrite the computed cells of each keyed row; add rows for new keys,
    report rows whose key is gone. `computed[key]` is {column: text}."""
    rows, span = doc.table(name)
    if rows is None:
        return
    head, body = rows[:2], rows[2:]
    key_of = key_of or (lambda r: r[0].strip("`"))
    seen = set()
    for r in body:
        k = key_of(r)
        if k not in computed:
            doc.problem("the {} table has a row for `{}`, which the tree does not have".format(name, k))
            continue
        seen.add(k)
        for col, val in computed[k].items():
            r[col] = val
    for k in keys:
        if k not in seen:
            body.append(new_row(k))
    doc.put_table(span, head + body)


# ── the checks ──────────────────────────────────────────────────────────────

def sync_readme(packs, subcommands, checks):
    doc = Doc(README)
    all_rules = [r for p in packs.values() for r in p["rules"]]
    total = len(all_rules)
    npm, pypi = package.NPM_PACKS, package.PYPI_PACKS
    doc.anchor(r"!\[rules: (\d+)\]\(https://img\.shields\.io/badge/rules-(\d+)-informational\)", [total, total])
    doc.anchor(r"\*\*Standard rules included\*\* \| (\d+) rules reimplemented", [total])
    doc.anchor(r"all (\w+) rule packs work today", [word(len(packs))])
    doc.anchor(r"(\w+) first-party packs: (\w+) on npm as `@codelaw/rules-<pack>`, (\w+) on PyPI",
               [word(len(packs), True), word(len(npm)), word(len(pypi))])
    doc.anchor(r"### Fix classes across all (\d+) rules", [total])
    doc.anchor(r"\| (\w+) rule packs: (\d+) rules \|", [word(len(packs), True), total])
    if checks is not None:
        doc.anchor(r"\*\*([\d,]+) example checks, all green\.\*\*", ["{:,}".format(checks)])

    def presets(name):
        return " · ".join("`{}`".format(p[0]) for p in packs[name]["presets"])

    computed = {n: {1: "✅" if n in npm else "—", 2: "✅" if n in pypi else "—",
                    3: str(len(packs[n]["rules"])), 4: presets(n)} for n in packs}
    sync_table(doc, "packs", list(packs), computed,
               lambda n: ["[`{0}`](packs/{0}/README.md)".format(n)] + [computed[n][c] for c in (1, 2, 3, 4)] + ["—"],
               key_of=lambda r: re.sub(r"^\[`([^`]+)`\].*$", r"\1", r[0]))
    counts = {c: sum(1 for r in all_rules if r[1] == c) for c in FIX_CLASSES}
    sync_table(doc, "fix-classes", list(FIX_CLASSES), {c: {1: str(counts[c])} for c in FIX_CLASSES},
               lambda c: ["`{}`".format(c), str(counts[c]), "—"])

    # roadmap: no row "designed" for a record built
    built = register_built()
    rows, _ = doc.table("roadmap")
    for r in (rows or [])[2:]:
        if len(r) < 4:
            doc.problem("roadmap row {!r} needs four cells: mark, feature, record, state".format(join_row(r)))
            continue
        names = re.findall(r"SR-[A-Z0-9-]+", r[2])
        if not names:
            doc.problem("roadmap row {!r} names no record".format(r[1]))
        for n in names:
            if n not in built:
                doc.problem("roadmap row {!r} names {}, which the register does not list".format(r[1], n))
            elif built[n] == "yes" and ("🔜" in r[0] or "designed" in r[3].lower()):
                doc.problem("roadmap row {!r} says it is not built, but the register marks {} `built: yes`".format(r[1], n))
    return doc


def sync_usage(subcommands):
    """USAGE.md: the from-source install loops and the command table (moved from README.md)."""
    doc = Doc(USAGE)
    npm, pypi = package.NPM_PACKS, package.PYPI_PACKS
    doc.anchor(r'for p in ([a-z -]+); do\n  ln -s "\$OLDPWD/codelaw/packs/\$p" "node_modules/', [" ".join(npm)])
    doc.anchor(r'for p in ([a-z -]+); do\n  ln -s "\$OLDPWD/codelaw/packs/\$p" "\.codelaw-packs/',
               [" ".join(p for p in pypi if p not in npm)])

    # commands: every subcommand has a row; no row names one that is gone
    rows, span = doc.table("commands")
    if rows is not None:
        documented = set()
        for r in rows[2:]:
            documented.update(re.findall(r"`codelaw ([a-z][a-z0-9-]*)", r[0]))
        names = [n for n, _ in subcommands]
        for gone in sorted(documented - set(names)):
            doc.problem("the command table names `codelaw {}`, which `codelaw --help` does not list".format(gone))
        missing = [(n, h) for n, h in subcommands if n not in documented]
        if missing:
            doc.put_table(span, rows + [["`codelaw {}`".format(n), h] for n, h in missing])

    return doc


def sync_pack_readmes(packs):
    docs = []
    for name, pack in packs.items():
        doc = Doc(ROOT / "packs" / name / "README.md")
        computed = {p: {1: str(c)} for p, c, _ in pack["presets"]}
        sync_table(doc, "presets", [p for p, _, _ in pack["presets"]], computed,
                   lambda p: ["`{}`".format(p), computed[p][1],
                              next(d for q, _, d in pack["presets"] if q == p) or "—"])
        rules = pack["rules"]
        doc.anchor(r"(\d+) rules: (\d+) `auto`, (\d+) `suggest`, (\d+) `manual`",
                   [len(rules)] + [sum(1 for r in rules if r[1] == c) for c in FIX_CLASSES], optional=True)
        doc.anchor(r"(\d+) rules are `native:`", [sum(1 for r in rules if r[2])], optional=True)
        docs.append(doc)
    return docs


def sync_packaging_readme():
    doc = Doc(PKG_README)
    rows, span = doc.table("registries")
    if rows is not None:
        def line(names):
            return " · ".join("`{}`".format(n) for n in names)
        doc.put_table(span, rows[:2] + [
            ["npm", "`@codelaw/rules-<name>`", line(package.NPM_PACKS)],
            ["PyPI", "`codelaw-rules-<name>`", line(package.PYPI_PACKS)],
        ])
    for name in api_names():
        if not re.search(r"\b{}\b".format(re.escape(name)), doc.text):
            doc.problem("the API exports `{}`, which this README does not name".format(name))
    return doc


def check_changelog():
    doc = Doc(CHANGELOG)
    parts = re.split(r"^## ", doc.text, flags=re.M)[1:]
    if not parts or not parts[0].startswith("Unreleased"):
        doc.problem("the first `## ` section is not `## Unreleased`")
        return doc
    unreleased = parts[0]
    release = next((p for p in parts[1:] if re.search(r"\d{4}-\d{2}-\d{2}", p.splitlines()[0])), None)
    if release is None:
        doc.problem("no release heading carries a date (`## 0.1.0 — YYYY-MM-DD`)")
        return doc
    day = re.search(r"\d{4}-\d{2}-\d{2}", release.splitlines()[0]).group(0)
    for line in IMPLEMENTATION.read_text(encoding="utf-8").splitlines():
        cells = split_row(line) if line.startswith("| 20") else []
        if len(cells) < 2 or cells[0] < day:
            continue
        for wid in re.findall(r"\bW-\d+\b", cells[1]):
            named = re.compile(r"\b{}\b".format(wid))
            if named.search(unreleased) or (cells[0] == day and named.search(release)):
                continue
            where = "*Unreleased*" if cells[0] > day else "*Unreleased* or the {} release".format(day)
            doc.problem("{} landed {} (work/IMPLEMENTATION.md) and {} does not name it".format(wid, cells[0], where))
    return doc


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="write nothing; exit 1 on drift")
    mode.add_argument("--write", action="store_true", help="rewrite every computed figure and table")
    ap.add_argument("--codelaw", metavar="BIN", help="read the subcommands from BIN --help and count the example checks")
    args = ap.parse_args()

    packs = read_packs()
    if args.codelaw:
        subcommands = subcommands_from_binary(args.codelaw)
        source = [n for n, _ in subcommands_from_source()]
        if [n for n, _ in subcommands] != source:
            sys.exit("docs-sync: {} --help lists {}, main.rs {}: rebuild the binary".format(
                args.codelaw, [n for n, _ in subcommands], source))
        checks = example_checks(args.codelaw)
    else:
        subcommands, checks = subcommands_from_source(), None

    docs = [sync_readme(packs, subcommands, checks), sync_usage(subcommands), sync_packaging_readme(), check_changelog()]
    docs += sync_pack_readmes(packs)
    failed = False
    for doc in docs:
        rel = doc.path.relative_to(ROOT)
        if doc.text != doc.original:
            if args.write:
                doc.path.write_text(doc.text, encoding="utf-8")
                print("wrote {}".format(rel))
            else:
                print("{}: out of date with the tree; run `python3 scripts/docs-sync.py --write`".format(rel))
                failed = True
        for p in doc.problems:
            print(p)
            failed = True
    if failed:
        return 1
    total = sum(len(p["rules"]) for p in packs.values())
    print("✔ docs-sync: {} packs, {} rules, {} commands{}".format(
        len(packs), total, len(subcommands), ", {:,} checks".format(checks) if checks is not None else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
