#!/usr/bin/env python3
"""The work system's mechanical checks — each one a rule its record states
and nothing else enforced (W-20). This script states no rule of its own:

  queue      SR-WORK-OPEN-QUEUE rules 13-17, 40, 42, 44, 45 and 64 — the
             shape of every row of work/OPEN-WORK.md and of its inbox (no
             stored `age`), and each row's item file's `ball:`
  lost       SR-WORK-OPEN-QUEUE veto — every `W-nn` named in a live document
             resolves to work/open/ or archive/open/
  ownership  SR-WORK-OWNERSHIP decisions 2 and 9 — each record's `owns:`
             and the register's Ownership table agree, in both directions;
             the table covers every top-level entry of the five folders and
             the workspace Cargo.toml; no `built: no` for code that exists
  registry   SR-WORK-REGISTRY decisions 5-8 — no row into archive/, every
             row's target exists, one row per document, every maintained
             document has its row
  archive    SR-WORK-ARCHIVE decision 1 — exactly one archive/, at the root

Read-only: it never writes a file. Exits 0 when every check passes; else
prints one line per violation and exits 1. `--self-test` runs it on a made-up
repo that breaks every rule once.
"""
import os
import re
import sys
import tempfile
from pathlib import Path

BALLS = ("🔴", "🟣", "🟡", "🟢")
ROW_RE = re.compile(
    r"^(?P<ball>🔴|🟣|🟡|🟢)(?:🧨)?(?:🔺)? \*\*(?P<id>W-\d+)\*\* `(?:arpit|agent|cowork)` .+"
    r" Filed by [^()]+ \([^()]+\)\. \[detail\]\((?P<link>open/(?P<link_id>W-\d+)-[A-Za-z0-9-]+\.md)\)$"
)
LINK_RE = re.compile(r"\]\(([^)\s]+)\)")
LANE_RE = re.compile(r"\*\* `(arpit|agent|cowork)` ")
COVERED = ("crates", "scripts", ".claude/hooks", "fixtures", "packs")
W_RE = re.compile(r"\bW-\d+\b")
SKIP_DIRS = {".git", "target", "node_modules", "worktrees"}


def section(lines, heading):
    """The lines under `heading` up to the next `---` line or `## ` heading."""
    try:
        start = next(i for i, l in enumerate(lines) if l.strip() == heading)
    except StopIteration:
        return None
    out = []
    for l in lines[start + 1:]:
        if l.strip() == "---" or l.startswith("## "):
            break
        out.append(l)
    return out


def links_exist(text, base, where, out):
    """Rule 45: a row points only at things that exist."""
    for target in LINK_RE.findall(text):
        if re.match(r"^[a-z]+:", target) or target.startswith("#"):
            continue
        path = target.split("#", 1)[0]
        if path and not (base / path).exists():
            out.append(f"{where}: links `{target}`, which does not exist (SR-WORK-OPEN-QUEUE rule 45)")


def check_queue(root):
    out = []
    path = root / "work" / "OPEN-WORK.md"
    if not path.is_file():
        return [f"work/OPEN-WORK.md is missing (SR-WORK-OPEN-QUEUE rule 1)"]
    lines = path.read_text(encoding="utf-8").splitlines()
    base = path.parent

    inbox = section(lines, "## Blocked on the decider")
    if inbox is None:
        out.append("work/OPEN-WORK.md has no `## Blocked on the decider` section (SR-WORK-OPEN-QUEUE rule 39)")
        inbox = []
    header = [l for l in inbox if l.startswith("|")][:1]
    if header and "age" in [c.strip().lower() for c in header[0].strip().strip("|").split("|")]:
        out.append("inbox: a stored `age` column; age is computed from `filed`, never stored (SR-WORK-OPEN-QUEUE rule 40)")
    table = [l for l in inbox if l.startswith("|")][2:]  # past the header and separator
    for i, l in enumerate(table):
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if cells[0].startswith("↳"):
            if not cells[0].startswith("↳ **blocks:**"):
                out.append(f"inbox row {i + 1}: a `↳` sub-row is `↳ **blocks:** …` (SR-WORK-OPEN-QUEUE rule 42)")
            continue
        what = cells[0][:60]
        if not cells[0].startswith("🔴"):
            out.append(f"inbox row `{what}…`: an inbox row is always 🔴 (SR-WORK-OPEN-QUEUE rule 44)")
        after = table[i + 1].strip().strip("|").strip() if i + 1 < len(table) else ""
        if not after.startswith("↳ **blocks:**"):
            out.append(f"inbox row `{what}…`: not followed by a `↳ **blocks:**` sub-row (SR-WORK-OPEN-QUEUE rule 42)")
        links_exist(l, base, f"inbox row `{what}…`", out)

    items = section(lines, "## Open items")
    if items is None:
        return out + ["work/OPEN-WORK.md has no `## Open items` section (SR-WORK-OPEN-QUEUE rule 1)"]
    rows = []  # (first line number, [physical lines])
    for n, l in enumerate(items):
        if not l.strip() or l.startswith("### "):
            rows.append(None)
            continue
        if l.startswith(BALLS):
            rows.append((n, [l]))
        elif rows and rows[-1] is not None and not l.startswith((" ", "\t", "-", "*", "|", ">")):
            rows[-1][1].append(l)  # the row's own second line
        else:
            out.append(f"Open items: `{l.strip()[:60]}` is a row body, or a row with no ball (SR-WORK-OPEN-QUEUE rules 16, 20)")
    for row in filter(None, rows):
        physical = row[1]
        text = " ".join(p.strip() for p in physical)
        m = ROW_RE.match(text)
        name = m.group("id") if m else text[:40]
        if len(physical) > 2:
            out.append(f"{name}: {len(physical)} lines; a row is one or two (SR-WORK-OPEN-QUEUE rule 13)")
        if not m:
            out.append(f"`{name}…`: not `ball · **W-nn** · `lane` · what · Filed by <person> (<surface>). · [detail](open/W-nn-slug.md)` (SR-WORK-OPEN-QUEUE rule 14)")
        # Counted as read: the text of a link, not its target; no `**` or backticks.
        shown = LINK_RE.sub("]", text).replace("[", "").replace("]", "").replace("**", "").replace("`", "")
        if len(shown) > 280:
            out.append(f"{name}: {len(shown)} characters as rendered; at most 280 (SR-WORK-OPEN-QUEUE rule 15)")
        if m and m.group("id") != m.group("link_id"):
            out.append(f"{name}: links {m.group('link')}, another item's file (SR-WORK-OPEN-QUEUE rule 17)")
        if m and (base / m.group("link")).is_file():
            want = "arpit" if m.group("ball") == "🔴" else LANE_RE.search(text).group(1)
            got = frontmatter((base / m.group("link")).read_text(encoding="utf-8")).get("ball")
            if got != want:
                out.append(f"{name}: its file says `ball: {got or '(none)'}`; the row makes it `{want}` (SR-WORK-OPEN-QUEUE rule 64)")
        links_exist(text, base, name, out)
    return out


def check_lost(root):
    """Every `W-nn` named in CLAUDE.md, work/, records/ or docs/ resolves."""
    out = []
    known = set()
    for d in ("work/open", "archive/open"):
        for f in (root / d).glob("W-*.md"):
            m = re.match(r"(W-\d+)-", f.name)
            if m:
                known.add(m.group(1))
    named = {}
    files = [root / "CLAUDE.md"] if (root / "CLAUDE.md").is_file() else []
    for d in ("work", "records", "docs"):
        files += sorted((root / d).rglob("*.md")) if (root / d).is_dir() else []
    for f in files:
        for w in W_RE.findall(f.read_text(encoding="utf-8")):
            named.setdefault(w, f.relative_to(root).as_posix())
    for w in sorted(set(named) - known, key=lambda x: int(x[2:])):
        out.append(f"lost: {w} (named in {named[w]}) has no file in work/open/ or archive/open/ (SR-WORK-OPEN-QUEUE veto)")
    return out


def frontmatter(text):
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---", 4)
    fields, key = {}, None
    for l in text[4:end].splitlines():
        m = re.match(r"^([A-Za-z_]+):\s*(.*)$", l)
        if m:
            key, value = m.group(1), m.group(2).strip()
            if value.startswith("[") and value.endswith("]"):
                fields[key] = [v.strip().strip("\"'") for v in value[1:-1].split(",") if v.strip()]
            else:
                fields[key] = value.strip("\"'") if value else []
        elif key and l.strip().startswith("- ") and isinstance(fields.get(key), list):
            fields[key].append(l.strip()[2:].strip().strip("\"'"))
    return fields


def marked_table(text, name):
    m = re.search(rf"<!-- {name}-START -->(.*?)<!-- {name}-END -->", text, re.S)
    rows = []
    for l in (m.group(1) if m else "").splitlines():
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if l.startswith("|") and len(cells) >= 2 and cells[0].startswith("`"):
            rows.append((cells[0].strip("`"), cells[1]))
    return rows, m is not None


def check_ownership(root):
    out = []
    register = root / "records" / "README.md"
    rows, found = marked_table(register.read_text(encoding="utf-8"), "OWNERSHIP-TABLE")
    if not found:
        return ["records/README.md has no OWNERSHIP-TABLE markers (SR-WORK-OWNERSHIP decision 2)"]
    table = {}
    for component, owner in rows:
        if component in table:
            out.append(f"Ownership table: `{component}` has two rows (SR-WORK-OWNERSHIP decision 1)")
        table[component] = owner
        if not (root / component).exists():
            out.append(f"Ownership table: `{component}` does not exist (SR-WORK-OWNERSHIP decision 2)")
    owns = {}
    for f in sorted((root / "records").glob("[0-9]*.md")):
        fm = frontmatter(f.read_text(encoding="utf-8"))
        name = fm.get("name")
        if isinstance(name, str) and name:
            owns[name] = (f.name, fm.get("owns") or [])
    for name, (file, components) in owns.items():
        for c in components:
            if c not in table:
                out.append(f"{name} ({file}) owns `{c}`, which the Ownership table does not list (SR-WORK-OWNERSHIP decision 2)")
            elif table[c] != name:
                out.append(f"{name} ({file}) owns `{c}`, which the Ownership table gives to {table[c]} (SR-WORK-OWNERSHIP decision 2)")
    for c, owner in table.items():
        if re.match(r"^W-\d+$", owner):
            # A component with no decision yet is owned by an open item (the register's own rule).
            if not list((root / "work" / "open").glob(owner + "-*.md")):
                out.append(f"Ownership table: `{c}` is owned by {owner}, which is no open item (SR-WORK-OWNERSHIP decision 2)")
        elif owner not in owns:
            out.append(f"Ownership table: `{c}` is owned by {owner}, which is no record's name (SR-WORK-OWNERSHIP decision 2)")
        elif c not in owns[owner][1]:
            out.append(f"Ownership table: `{c}` is {owner}'s, and {owns[owner][0]}'s `owns:` does not list it (SR-WORK-OWNERSHIP decision 2)")
    entries = ["Cargo.toml"] if (root / "Cargo.toml").is_file() else []
    for d in COVERED:
        if (root / d).is_dir():
            entries += sorted(f"{d}/{x.name}" for x in (root / d).iterdir() if not x.name.startswith(".") and x.name != "__pycache__")
    for e in entries:
        parts = e.split("/")
        if not any("/".join(parts[:k]) in table for k in range(1, len(parts) + 1)):
            out.append(f"Ownership table: `{e}` has no row and sits under none (SR-WORK-OWNERSHIP decision 2)")
    for l in register.read_text(encoding="utf-8").splitlines():
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if l.startswith("| [") and len(cells) == 5 and cells[4] == "no":
            name = cells[1].strip("*")
            comps = owns.get(name, ("", []))[1]
            if comps and all((root / c).exists() for c in comps):
                out.append(f"register: {name} says `built: no`, and every component it owns exists (SR-WORK-OWNERSHIP decision 9)")
    return out


def maintained(root):
    """The documents decision 8 asks a row for: the Markdown at the root,
    at the top of docs/ and work/, each work/*/ index, and the register
    and template of records/."""
    out = [p for p in root.glob("*.md")]
    out += list((root / "docs").glob("*.md")) + list((root / "work").glob("*.md"))
    out += list((root / "work").glob("*/README.md"))
    out += [root / "records" / n for n in ("README.md", "TEMPLATE.md") if (root / "records" / n).is_file()]
    return sorted(p.relative_to(root).as_posix() for p in out)


def check_registry(root):
    out = []
    record = root / "records" / "0060_WORK-registry.md"
    if not record.is_file():
        return ["records/0060_WORK-registry.md is missing (SR-WORK-REGISTRY decision 1)"]
    lines = record.read_text(encoding="utf-8").splitlines()
    rows, inside = [], False
    for l in lines:
        if l.startswith("## §3"):
            inside = True
        elif l.startswith("## ") and inside:
            break
        elif inside:
            m = re.match(r"^\| `([^`]+)` \|", l)
            if m:
                rows.append(m.group(1))
    seen = set()
    for p in rows:
        if p.startswith("archive/"):
            out.append(f"registry: `{p}` points into archive/ (SR-WORK-REGISTRY decision 5)")
        elif not (root / p).exists():
            out.append(f"registry: `{p}` does not exist (SR-WORK-REGISTRY decision 6)")
        if p in seen:
            out.append(f"registry: `{p}` has two rows (SR-WORK-REGISTRY decision 7)")
        seen.add(p)
    for p in maintained(root):
        if p not in seen:
            out.append(f"registry: `{p}` is a maintained document with no row (SR-WORK-REGISTRY decision 8)")
    return out


def check_archive(root):
    out = []
    for d, dirs, _ in os.walk(root):
        here = Path(d)
        if here == root:
            dirs[:] = [x for x in dirs if x != "archive" and x not in SKIP_DIRS]
        else:
            dirs[:] = [x for x in dirs if x not in SKIP_DIRS]
        for x in dirs:
            if x.lower() == "archive":
                out.append(f"a second archive: {(here / x).relative_to(root).as_posix()}/ (SR-WORK-ARCHIVE decision 1)")
    if not (root / "archive").is_dir():
        out.append("archive/ is missing at the repo root (SR-WORK-ARCHIVE decision 1)")
    return out


CHECKS = (check_queue, check_lost, check_ownership, check_registry, check_archive)


def run(root):
    out = []
    for check in CHECKS:
        out += check(root)
    return out


def self_test():
    """A made-up repo that breaks each rule once; every break must be named."""
    with tempfile.TemporaryDirectory() as t:
        r = Path(t)
        files = {
            "CLAUDE.md": "See W-01 and W-02 and W-99.\n",
            "work/open/W-01-a.md": "---\nball: arpit\n---\n# W-01\n",
            "archive/open/W-02-b.md": "# W-02\n",
            "work/README.md": "map\n",
            "work/NEW.md": "unregistered\n",
            "docs/old/archive/x.md": "a second archive\n",
            "records/README.md": "| [0001](0001_a.md) | **SR-A** | a | accepted | no |\n<!-- OWNERSHIP-TABLE-START -->\n| component | owner | note |\n|---|---|---|\n"
            "| `scripts/a.sh` | SR-A | ok |\n| `scripts/b.sh` | SR-A | not in owns |\n| `scripts/gone.sh` | SR-NONE | x |\n"
            "<!-- OWNERSHIP-TABLE-END -->\n",
            "records/0001_a.md": "---\nname: SR-A\nowns:\n  - scripts/a.sh\n  - scripts/c.sh\n---\n",
            "records/0060_WORK-registry.md": "## §3 — The registry\n\n| Document | t | v | n |\n|---|---|---|---|\n"
            "| `work/README.md` | t | v | n |\n| `work/README.md` | t | v | n |\n| `archive/open/W-02-b.md` | t | v | n |\n"
            "| `work/MISSING.md` | t | v | n |\n| `CLAUDE.md` | t | v | n |\n| `records/README.md` | t | v | n |\n"
            "| `work/OPEN-WORK.md` | t | v | n |\n\n## References\n",
            "scripts/a.sh": "", "scripts/b.sh": "", "scripts/c.sh": "", "packs/x/rule.yml": "",
            "work/OPEN-WORK.md": "# OPEN-WORK\n\n---\n\n## Blocked on the decider\n\n| what | filed | age |\n|---|---|---|\n"
            "| Decide a thing ([W-01](open/W-01-a.md)) | 2026-10-20 | 0 d |\n"
            "| 🔴 Decide another ([x](open/nope.md)) | 2026-10-20 | 0 d |\n| ↳ **blocks:** W-01 | | |\n\n---\n\n## Open items\n\n### build\n\n"
            "🟢 **W-01** `agent` Fine. Filed by Arpit (Cowork). [detail](open/W-01-a.md)\n"
            "🟢 **W-01** `agent` Wrong file. Filed by Arpit (Cowork). [detail](open/W-02-b.md)\n"
            "🟢 **W-01** `agent` " + "long " * 60 + "Filed by Arpit (Cowork). [detail](open/W-01-a.md)\n"
            "- a body under it\n"
            "🟢 W-01 no shape\n\n---\n",
        }
        for p, text in files.items():
            (r / p).parent.mkdir(parents=True, exist_ok=True)
            (r / p).write_text(text, encoding="utf-8")
        got = "\n".join(run(r))
        expect = [
            "rule 44", "rule 42", "open/nope.md", "rule 17", "rule 15", "rules 16, 20", "rule 14",
            "lost: W-99", "owns `scripts/c.sh`", "`scripts/b.sh` is SR-A's", "SR-NONE, which is no record",
            "`scripts/gone.sh` does not exist", "points into archive/", "`work/MISSING.md` does not exist",
            "`work/README.md` has two rows", "`work/NEW.md` is a maintained document", "a second archive: docs/old/archive/",
            "rule 40", "rule 64", "`packs/x` has no row", "decision 9",
        ]
        missed = [e for e in expect if e not in got]
        if missed or "W-01 (named" in got or "W-02 (named" in got:
            print("self-test failed; missed: " + repr(missed) + "\n" + got)
            return 1
    print(f"self-test: {len(expect)} breaks, each named")
    return 0


def main():
    if sys.argv[1:] == ["--self-test"]:
        return self_test()
    root = Path(__file__).resolve().parent.parent
    out = run(root)
    for line in out:
        print(line)
    return 1 if out else 0


if __name__ == "__main__":
    sys.exit(main())
