#!/usr/bin/env python3
"""How many people want each ported rule (W-140, SR-RULE-AUTHORING decision 2).

Two signals, both public and both sent by people who chose to:

- **code** — `.codelaw/rules/**.md` files on public GitHub whose
  `ported_from:` names a source rule: projects that ported it themselves
  with the `codelaw-port-rule` skill (`gh search code`);
- **issues** — issues on arpitarya/codelaw titled `port: <tool>/<rule>` or
  labelled `ported-rule`: the skill's opt-in share prompt (`gh search issues`).

One row per source rule: the repositories that carry a port, the issues that
ask for it, and the first date either was seen. The rules most wanted are
the candidates for a first-party pack (W-139).

Maintainer-run: it calls `gh` (logged in), is never run in CI and is no part
of the engine (L5). Standard library only (L11).

    scripts/ported-rule-demand.py                  # text table, from GitHub
    scripts/ported-rule-demand.py --format json    # the same, as JSON
    scripts/ported-rule-demand.py --dates          # also each file's first commit (one call per file)
    scripts/ported-rule-demand.py --save raw.json  # keep what gh returned
    scripts/ported-rule-demand.py --from-json raw.json   # offline, from a saved run
    scripts/ported-rule-demand.py --self-test      # offline check of the parsing and the table
"""
import argparse
import json
import re
import subprocess
import sys

REPO = "arpitarya/codelaw"
LABEL = "ported-rule"
TITLE_RE = re.compile(r"^\s*port:\s*([A-Za-z0-9@._-]+(?:/[A-Za-z0-9@._-]+)*/[A-Za-z0-9@._-]+)\s*$", re.I)
INLINE_RE = re.compile(r"^ported_from:\s*\[([^\]]*)\]", re.M)
BLOCK_RE = re.compile(r"^ported_from:\s*\n((?:[ \t]+-[^\n]*\n?)+)", re.M)
ISSUE_FIELD_RE = re.compile(r"^### Ported from\s*\n+([^\n#][^\n]*)", re.M)


def ported_from(text):
    """The `<tool>/<rule>` ids a rule file's `ported_from:` lists, inline or as a block."""
    ids = []
    for m in INLINE_RE.finditer(text):
        ids += [x.strip().strip("'\"") for x in m.group(1).split(",")]
    for m in BLOCK_RE.finditer(text):
        ids += [l.strip()[1:].strip().strip("'\"") for l in m.group(1).splitlines() if l.strip().startswith("-")]
    return sorted({i for i in ids if "/" in i})


def issue_source(issue):
    """The source rule an issue asks for: its `port:` title, else the form's field."""
    m = TITLE_RE.match(issue.get("title") or "")
    if m:
        return m.group(1)
    m = ISSUE_FIELD_RE.search(issue.get("body") or "")
    if m and "/" in m.group(1) and m.group(1).strip() != "_No response_":
        return m.group(1).strip().strip("`")
    return None


def table(raw):
    """Rows, most wanted first: {source, repos, issues, first_seen, repo_list, issue_list}."""
    rows = {}

    def row(source):
        return rows.setdefault(source, {"source": source, "repos": set(), "issues": set(), "dates": []})

    for f in raw.get("code", []):
        if f.get("repo") == REPO:
            continue  # codelaw's own fixtures are not demand
        for source in f.get("ported_from") or ported_from(f.get("text", "")):
            r = row(source)
            r["repos"].add(f["repo"])
            if f.get("first_seen"):
                r["dates"].append(f["first_seen"][:10])
    for i in raw.get("issues", []):
        source = issue_source(i)
        if source:
            r = row(source)
            r["issues"].add(i["url"])
            if i.get("created"):
                r["dates"].append(i["created"][:10])
    out = [
        {
            "source": r["source"],
            "repos": len(r["repos"]),
            "issues": len(r["issues"]),
            "first_seen": min(r["dates"]) if r["dates"] else None,
            "repo_list": sorted(r["repos"]),
            "issue_list": sorted(r["issues"]),
        }
        for r in rows.values()
    ]
    out.sort(key=lambda r: (-(r["repos"] + r["issues"]), -r["repos"], r["source"]))
    return out


def text(rows):
    if not rows:
        return "no ported rules found\n"
    w = max(len("source rule"), *(len(r["source"]) for r in rows))
    lines = [f"{'source rule':<{w}}  repos  issues  first seen"]
    for r in rows:
        lines.append(f"{r['source']:<{w}}  {r['repos']:>5}  {r['issues']:>6}  {r['first_seen'] or '-'}")
    return "\n".join(lines) + "\n"


# ── fetching, through gh ──────────────────────────────────────────────────


def gh(args):
    try:
        out = subprocess.run(["gh", *args], check=True, capture_output=True, text=True)
    except FileNotFoundError:
        sys.exit("ported-rule-demand: `gh` is not installed (https://cli.github.com)")
    except subprocess.CalledProcessError as e:
        sys.exit(f"ported-rule-demand: gh {' '.join(args)} failed: {e.stderr.strip()}")
    return json.loads(out.stdout or "null")


def fetch(limit, dates):
    hits = gh(["search", "code", "ported_from", "--filename", "*.md", "--match", "file", "--limit", str(limit),
               "--json", "repository,path,textMatches"]) or []
    code = []
    for h in hits:
        path = h.get("path", "")
        if "/.codelaw/rules/" not in "/" + path:
            continue
        repo = h["repository"]["nameWithOwner"]
        fragments = "\n".join(m.get("fragment", "") for m in h.get("textMatches") or [])
        ids = ported_from(fragments)
        if not ids:  # the fragment cut the line: read the file
            raw = subprocess.run(["gh", "api", f"repos/{repo}/contents/{path}", "-H", "Accept: application/vnd.github.raw+json"],
                                 capture_output=True, text=True)
            ids = ported_from(raw.stdout) if raw.returncode == 0 else []
        first = None
        if dates and ids:
            commits = gh(["api", f"repos/{repo}/commits?path={path}&per_page=100"]) or []
            if commits:
                first = commits[-1]["commit"]["committer"]["date"]
        code.append({"repo": repo, "path": path, "ported_from": ids, "first_seen": first})
    issues = {}
    for extra in (["--", "port: in:title"], ["--label", LABEL]):
        for i in gh(["search", "issues", "--repo", REPO, "--limit", str(limit), "--json", "title,url,createdAt,body", *extra]) or []:
            issues[i["url"]] = {"url": i["url"], "title": i["title"], "created": i.get("createdAt"), "body": i.get("body", "")}
    return {"code": code, "issues": sorted(issues.values(), key=lambda i: i["url"])}


# ── the self-test: no network ─────────────────────────────────────────────

SAMPLE = {
    "code": [
        {"repo": "a/web", "path": ".codelaw/rules/a/no-nested-ternary.md",
         "text": "---\nid: a/no-nested-ternary\nported_from: [eslint/no-nested-ternary]\n---\n", "first_seen": "2026-11-02T10:00:00Z"},
        {"repo": "b/app", "path": "tools/.codelaw/rules/b/ternary.md",
         "text": "ported_from:\n  - eslint/no-nested-ternary\n  - 'biome/noNestedTernary'\n"},
        {"repo": "b/app", "path": ".codelaw/rules/b/glob.md", "ported_from": ["pylint/global-statement"]},
        {"repo": REPO, "path": "fixtures/port-rule/.codelaw/rules/acme/no-important.md",
         "ported_from": ["stylelint/declaration-no-important"]},
    ],
    "issues": [
        {"url": "u/1", "title": "port: eslint/no-nested-ternary", "created": "2026-10-31T09:00:00Z"},
        {"url": "u/2", "title": "[rule] no !important", "created": "2026-11-05T09:00:00Z",
         "body": "### Ported from\n\nstylelint/declaration-no-important\n\n### The ported rule file\n"},
        {"url": "u/3", "title": "[rule] something", "body": "### Ported from\n\n_No response_\n"},
    ],
}


def self_test():
    rows = table(SAMPLE)
    got = [(r["source"], r["repos"], r["issues"], r["first_seen"]) for r in rows]
    want = [
        ("eslint/no-nested-ternary", 2, 1, "2026-10-31"),
        ("biome/noNestedTernary", 1, 0, None),
        ("pylint/global-statement", 1, 0, None),
        ("stylelint/declaration-no-important", 0, 1, "2026-11-05"),
    ]
    assert got == want, got
    assert ported_from("ported_from: [ruff/F401, \"eslint/no-unused-vars\"]") == ["eslint/no-unused-vars", "ruff/F401"]
    assert issue_source({"title": "port: eslint-plugin-react/jsx-key"}) == "eslint-plugin-react/jsx-key"
    assert text([]) == "no ported rules found\n"
    assert text(rows).splitlines()[1].split() == ["eslint/no-nested-ternary", "2", "1", "2026-10-31"]
    json.dumps(rows)
    print("ported-rule-demand: self-test passed")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--format", choices=["text", "json"], default="text")
    ap.add_argument("--limit", type=int, default=1000, help="results per gh search (GitHub caps code search at 1000)")
    ap.add_argument("--dates", action="store_true", help="each file's first commit date (one API call per file)")
    ap.add_argument("--from-json", metavar="FILE", help="read a saved run instead of calling gh")
    ap.add_argument("--save", metavar="FILE", help="write what gh returned, for --from-json")
    ap.add_argument("--self-test", action="store_true", help="check the parsing and the table offline")
    a = ap.parse_args()
    if a.self_test:
        return self_test()
    if a.from_json:
        with open(a.from_json, encoding="utf-8") as f:
            raw = json.load(f)
    else:
        raw = fetch(a.limit, a.dates)
    if a.save:
        with open(a.save, "w", encoding="utf-8") as f:
            json.dump(raw, f, indent=2)
    rows = table(raw)
    if a.format == "json":
        print(json.dumps({"repository": REPO, "rows": rows}, indent=2))
    else:
        sys.stdout.write(text(rows))


if __name__ == "__main__":
    main()
