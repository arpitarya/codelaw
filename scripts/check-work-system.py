#!/usr/bin/env python3
"""The work-system checks the WORK records name and nothing ran (W-20 DoD 5).

Each check is the executable form of one rule stated in a record; this script
states no rule of its own:

  queue-rows   SR-WORK-OPEN-QUEUE rules 13-17 (and 18): an Open items row is one
               or two lines, has the rule-14 shape, is at most 280 characters,
               has no body, and links its own detail file.
  inbox        SR-WORK-OPEN-QUEUE rule 42: every inbox row is followed by its
               `↳ **blocks:**` sub-row; rule 45a by check-open-work-inbox.py.
  links        SR-WORK-OPEN-QUEUE rule 45: every link in the queue resolves.
  lost-ids     SR-WORK-OPEN-QUEUE's veto: every `W-nn` named in CLAUDE.md,
               work/, records/ or docs/ resolves to work/open/ or archive/open/.
  ownership    SR-WORK-OWNERSHIP decision 2: every record's `owns:` and the
               register's Ownership table agree in both directions, each
               component is listed once, and every file under the owned trees
               has an owner (records/README.md §Ownership).
  registry     SR-WORK-REGISTRY decisions 5-8: no row points into archive/,
               every target exists, one row per document, and every
               maintained document has a row.
  archive      SR-WORK-ARCHIVE decision 1: exactly one `archive/`, at the root.

Read-only. Exits 0 with one summary line when everything holds; exits 1 with
one line per violation otherwise. Python 3.9 syntax (work/MACHINE.md).

    python3 scripts/check-work-system.py
"""
import importlib.util
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BALLS = "🔴🟣🟡🟢"
LANES = ("arpit", "agent", "cowork")  # SR-WORK-OPEN-QUEUE rule 35
ROW_MAX = 280  # rule 15
ROW_RE = re.compile(
    r"^[🔴🟣🟡🟢]🧨?🔺? \*\*(W-\d+)\*\* `([a-z]+)` .+ "
    r"Filed by [^()]+ \([^()]+\)\. \[detail\]\(open/(W-\d+)-[a-z0-9-]+\.md\)$"
)
LINK_RE = re.compile(r"\]\(([^)\s]+)\)")
ID_RE = re.compile(r"\bW-[0-9]+\b")
# The trees whose every file has an owner (records/README.md §Ownership).
OWNED_TREES = ("scripts/", ".claude/hooks/", "crates/", "fixtures/", "packs/")
# Documents the registry does not track, each with the rule that keeps it
# current by other means (SR-WORK-REGISTRY decision 8 names *maintained* ones).
NOT_MAINTAINED = {
    "work/NOW.md": "one line, overwritten by every session (SR-WORK-SESSION decision 5)",
    "work/WORKLOG.md": "append-only, one entry per session (SR-WORK-SESSION)",
}


def rel(p):
    return p.relative_to(ROOT).as_posix()


def section(lines, heading):
    """The lines under `heading` up to the next `---` or `## ` heading."""
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


def check_queue(errors):
    queue = ROOT / "work" / "OPEN-WORK.md"
    lines = queue.read_text(encoding="utf-8").splitlines()
    where = "work/OPEN-WORK.md"

    # Open items: rules 13-17.
    items = section(lines, "## Open items")
    if items is None:
        errors.append(f"{where}: no '## Open items' section")
        items = []
    rows = []
    for l in items:
        if not l.strip() or l.startswith("### "):
            continue
        if l[0] in BALLS:
            rows.append([l])
        elif l[0] in " \t" or re.match(r"^([-*+]|\d+\.) ", l) or l.startswith("|"):
            errors.append(f"{where}: rule 16 — a row has a body (nested bullet, table or indented line): {l.strip()[:60]!r}")
        elif rows:
            rows[-1].append(l)
        else:
            errors.append(f"{where}: rule 20 — a line with no ball: {l[:60]!r}")
    for r in rows:
        text = " ".join(x.strip() for x in r)
        head = text[:50]
        if len(r) > 2:
            errors.append(f"{where}: rule 13 — a row of {len(r)} lines: {head!r}")
        if len(text) > ROW_MAX:
            errors.append(f"{where}: rule 15 — a row of {len(text)} characters (at most {ROW_MAX}): {head!r}")
        m = ROW_RE.match(text)
        if not m:
            errors.append(f"{where}: rule 14 — not `ball · **W-nn** · `lane` · what · Filed by <person> (<surface>). · [detail](open/W-nn-slug.md)`: {head!r}")
            continue
        if m.group(2) not in LANES:
            errors.append(f"{where}: rule 35 — lane `{m.group(2)}` is not one of {', '.join(LANES)}: {head!r}")
        if m.group(1) != m.group(3):
            errors.append(f"{where}: rule 17 — row {m.group(1)} links {m.group(3)}'s file")
    if not rows:
        errors.append(f"{where}: no Open items rows were read — the parser and the queue disagree")

    # The inbox: rules 18 and 42 (45a is check-open-work-inbox.py's).
    inbox = section(lines, "## Blocked on the decider")
    table = [l for l in (inbox or []) if l.startswith("|")][2:]
    for i, l in enumerate(table):
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if cells[0].startswith("↳"):
            if not cells[0].startswith("↳ **blocks:**"):
                errors.append(f"{where}: rule 42 — a sub-row that is not `↳ **blocks:**`: {cells[0][:60]!r}")
            elif i == 0 or table[i - 1].strip().strip("|").strip().startswith("↳"):
                errors.append(f"{where}: rule 42 — a `↳ **blocks:**` sub-row with no inbox row above it")
            continue
        if len(cells[0]) > ROW_MAX:
            errors.append(f"{where}: rule 18 — an inbox row of {len(cells[0])} characters (at most {ROW_MAX}): {cells[0][:50]!r}")
        nxt = table[i + 1] if i + 1 < len(table) else ""
        if not nxt.strip().strip("|").strip().startswith("↳ **blocks:**"):
            errors.append(f"{where}: rule 42 — inbox row not followed by its `↳ **blocks:**` sub-row: {cells[0][:50]!r}")

    # Rule 45: every link points at something that exists.
    for n, l in enumerate(lines, 1):
        for target in LINK_RE.findall(l):
            if re.match(r"^[a-z]+:", target) or target.startswith("#"):
                continue
            path = (queue.parent / target.split("#")[0]).resolve()
            if not path.exists():
                errors.append(f"{where}:{n}: rule 45 — the link `{target}` points at nothing")

    # Rule 45a, through its own script.
    sys.dont_write_bytecode = True  # read-only: no __pycache__ in scripts/
    spec = importlib.util.spec_from_file_location("inbox", ROOT / "scripts" / "check-open-work-inbox.py")
    inbox_mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(inbox_mod)
    e = inbox_mod.check(queue)
    if e:
        errors.append(f"{where}: {e}")


def markdown_files():
    out = [ROOT / "CLAUDE.md"]
    for top in ("work", "records", "docs"):
        out += sorted((ROOT / top).rglob("*.md"))
    return [p for p in out if p.is_file()]


def check_lost_ids(errors):
    """SR-WORK-OPEN-QUEUE's veto, done right: an id resolves when EITHER folder has it."""
    homes = set()
    for d in ("work/open", "archive/open"):
        for p in (ROOT / d).glob("W-*.md"):
            m = re.match(r"(W-\d+)-", p.name)
            if m:
                homes.add(m.group(1))
    seen = {}
    for p in markdown_files():
        for n, l in enumerate(p.read_text(encoding="utf-8").splitlines(), 1):
            for w in ID_RE.findall(l):
                seen.setdefault(w, f"{rel(p)}:{n}")
    if not seen:
        errors.append("lost ids: no `W-nn` was found anywhere — the scan is broken")
    for w in sorted(seen):
        if w not in homes:
            errors.append(f"{seen[w]}: lost id — {w} is in neither work/open/ nor archive/open/")


def table_rows(text, start, end):
    m = re.search(re.escape(start) + r"(.*?)" + re.escape(end), text, re.S)
    rows = []
    for l in (m.group(1) if m else "").splitlines():
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if not l.startswith("|") or len(cells) < 2 or cells[0] in ("component",) or set(cells[0]) <= set("-"):
            continue
        comp = cells[0].strip("`").split("::")[0].rstrip("/")
        rows.append((comp, cells[1].strip("*` ")))
    return rows, m is not None


def frontmatter_owns(text):
    m = re.match(r"---\n(.*?)\n---\n", text, re.S)
    if not m:
        return None, []
    fm = m.group(1)
    name = re.search(r"^name:\s*(\S+)", fm, re.M)
    owns = []
    block = re.search(r"^owns:(.*?)(?=^\S)", fm + "\nend:", re.M | re.S)
    if block:
        inline = block.group(1).strip()
        if inline.startswith("["):
            owns = [x.strip().strip("'\"") for x in inline.strip("[]").split(",") if x.strip()]
        else:
            owns = [x.strip()[2:].strip().strip("'\"").rstrip("/") for x in block.group(1).splitlines() if x.strip().startswith("- ")]
    return (name.group(1) if name else None), owns


def tracked_files():
    try:
        out = subprocess.run(["git", "ls-files", "-z"], cwd=ROOT, capture_output=True, check=True).stdout
        return [f for f in out.decode("utf-8").split("\0") if f]
    except (OSError, subprocess.CalledProcessError):
        files = []
        for d, dirs, names in os.walk(ROOT):
            dirs[:] = [x for x in dirs if x not in (".git", "target", "node_modules", "__pycache__")]
            files += [Path(d, n).relative_to(ROOT).as_posix() for n in names]
        return files


def check_ownership(errors):
    register = (ROOT / "records" / "README.md").read_text(encoding="utf-8")
    rows, found = table_rows(register, "<!-- OWNERSHIP-TABLE-START -->", "<!-- OWNERSHIP-TABLE-END -->")
    where = "records/README.md §Ownership"
    if not found or not rows:
        errors.append(f"{where}: the marked table is missing")
        return
    table = {}
    for comp, owner in rows:
        if comp in table:
            errors.append(f"{where}: `{comp}` is listed twice")
        table[comp] = owner
    declared = {}
    for p in sorted((ROOT / "records").glob("[0-9][0-9][0-9][0-9]_*.md")):
        name, owns = frontmatter_owns(p.read_text(encoding="utf-8"))
        if name is None:
            continue
        declared[name] = p
        for comp in owns:
            if table.get(comp) != name:
                got = f"the table says {table[comp]}" if comp in table else "the table does not list it"
                errors.append(f"{rel(p)}: decision 2 — `owns: {comp}`, but {got}")
    for comp, owner in table.items():
        if owner.startswith("W-"):
            if not list((ROOT / "work" / "open").glob(f"{owner}-*.md")):
                errors.append(f"{where}: `{comp}` is owned by {owner}, which is not an open item")
            continue
        if owner not in declared:
            errors.append(f"{where}: `{comp}` is owned by {owner}, which is no record")
            continue
        _, owns = frontmatter_owns(declared[owner].read_text(encoding="utf-8"))
        if comp not in owns:
            errors.append(f"{rel(declared[owner])}: decision 2 — the table gives it `{comp}`, but its `owns:` does not list it")
        if not (ROOT / comp).exists():
            errors.append(f"{where}: `{comp}` does not exist")
    # Every file under the owned trees has an owner (most specific wins).
    for f in tracked_files():
        if f.startswith(OWNED_TREES) and not any(f == c or f.startswith(c + "/") for c in table):
            errors.append(f"{where}: `{f}` has no owner")


def check_registry(errors):
    record = ROOT / "records" / "0060_WORK-registry.md"
    lines = record.read_text(encoding="utf-8").splitlines()
    where = "records/0060_WORK-registry.md §3"
    body = []
    on = False
    for l in lines:
        if l.startswith("## §3"):
            on = True
        elif l.startswith("## ") and on:
            break
        elif on:
            body.append(l)
    docs = []
    for l in body:
        m = re.match(r"^\| `([^`]+)` \|", l)
        if m:
            docs.append(m.group(1))
    if not docs:
        errors.append(f"{where}: no rows were read")
    seen = set()
    for d in docs:
        if d.startswith("archive/"):
            errors.append(f"{where}: decision 5 — `{d}` points into archive/")
        if not (ROOT / d).exists():
            errors.append(f"{where}: decision 6 — `{d}` does not exist")
        if d in seen:
            errors.append(f"{where}: decision 7 — `{d}` has two rows")
        seen.add(d)
    # Decision 8: the maintained documents — the root's, docs/, work/ and the
    # READMEs of work/'s folders, the register and the template.
    maintained = [p for p in ROOT.glob("*.md")] + list((ROOT / "docs").glob("*.md")) + list((ROOT / "work").glob("*.md"))
    maintained += list((ROOT / "work").glob("*/README.md")) + [ROOT / "records" / "README.md", ROOT / "records" / "TEMPLATE.md"]
    for p in sorted(set(maintained)):
        r = rel(p)
        if p.is_file() and r not in seen and r not in NOT_MAINTAINED:
            errors.append(f"{where}: decision 8 — `{r}` is a maintained document with no row")
    for r in NOT_MAINTAINED:
        if not (ROOT / r).exists():
            errors.append(f"scripts/check-work-system.py: NOT_MAINTAINED names `{r}`, which does not exist")


def check_archive(errors):
    found = []
    for d, dirs, _ in os.walk(ROOT):
        dirs[:] = [x for x in dirs if x not in (".git", "node_modules", "target")]
        for x in dirs:
            if x == "archive":
                found.append(Path(d, x).relative_to(ROOT).as_posix())
    if found != ["archive"]:
        errors.append(f"SR-WORK-ARCHIVE decision 1 — exactly one `archive/`, at the root; found: {', '.join(found) or 'none'}")


def main():
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    checks = [
        ("queue rows, inbox, links (SR-WORK-OPEN-QUEUE rules 13-18, 42, 45, 45a)", check_queue),
        ("lost W-nn ids (SR-WORK-OPEN-QUEUE veto)", check_lost_ids),
        ("owns ↔ Ownership table (SR-WORK-OWNERSHIP decision 2)", check_ownership),
        ("the registry (SR-WORK-REGISTRY decisions 5-8)", check_registry),
        ("one archive/ (SR-WORK-ARCHIVE decision 1)", check_archive),
    ]
    errors = []
    for _, fn in checks:
        fn(errors)
    if errors:
        for e in errors:
            print(f"✖ {e}")
        print(f"\n{len(errors)} work-system violation(s)")
        return 1
    print(f"✔ the work system holds: {'; '.join(name for name, _ in checks)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
