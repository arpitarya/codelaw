---
id: data/codeowners-coverage
title: "Every file has a code owner, and every CODEOWNERS pattern is live"
languages: [json, toml]
files: ["package.json", "pyproject.toml", "codelaw.toml"]
fix: manual
fix_reason: "who owns a path is a decision about people; the finding names the paths and the lines, and the owner is the team's to choose"
stability: preview
options:
  path: { type: "string", description: "the CODEOWNERS file, from the project root; by default the first that exists of .github/CODEOWNERS, CODEOWNERS, docs/CODEOWNERS and .gitlab/CODEOWNERS" }
  allow_unowned: { type: "boolean", default: false, description: "a pattern that names no owner is meant: it is not reported, and neither are its paths" }
  ignore: { type: "string[]", default: [], description: "gitignore-style patterns of paths that need no owner, such as *.lock" }
since: 0.5.0
---

CODEOWNERS routes each change to the people who should review it, so a file no pattern owns is a change nobody is asked about. This rule reads the project's CODEOWNERS file and its files — never a network — and reports three things: a file no pattern owns, a pattern that matches no file (stale), and a pattern that names no owner. The file is `[codeowners] path`, else the first that exists of `.github/CODEOWNERS`, `CODEOWNERS`, `docs/CODEOWNERS` and `.gitlab/CODEOWNERS`; a project with none gets no finding. GitHub's and GitLab's syntax both work: gitignore-style patterns, the last match wins, and GitLab's `[Section]` lines start a section (a file is owned when any section owns it).

CODEOWNERS is not one of codelaw's languages, so the findings sit on the project's root manifest — `package.json`, else `pyproject.toml`, else `codelaw.toml` — once per project, each naming the CODEOWNERS line. Unowned files are one finding that names up to ten paths and the count.

A pattern with no owner takes its paths out of ownership on purpose in GitHub's syntax; it is reported unless `[codeowners] allow_unowned = true`, which also keeps its paths out of the unowned list. `[codeowners] ignore` lists gitignore-style patterns of paths that need no owner, such as `*.lock`.

**Why:** A path with no owner is reviewed by whoever happens to look, and a stale pattern keeps an owner on the hook for code that moved, so the next file that lands there is owned by nobody and nothing says so.

```codelaw
native: data/codeowners-coverage
```

## How to fix

1. For an unowned path, add a pattern that covers it to the CODEOWNERS file (`/src/payments/ @org/payments`), or a catch-all `* @org/maintainers` first so every later pattern refines it.
2. For a stale pattern, delete the line, or correct the pattern to the folder or file it now means.
3. For a pattern with no owner, name one — or, if the path is meant to have none, set `[codeowners] allow_unowned = true`.
4. For paths that never need a review (lockfiles, generated files), list them in `[codeowners] ignore`.

## Examples

### JSON

#### Don't

```json
// package.json
{
  "name": "app"
}
// src/a.js
// CODEOWNERS
/src/ @org/app
```

#### Do

```json
// package.json
{
  "name": "app"
}
// src/a.js
// CODEOWNERS
* @org/app
```

### TOML

#### Don't

```toml
# pyproject.toml
[project]
name = "app"
# src/a.py
# CODEOWNERS
/src/ @org/app
```

#### Do

```toml
# pyproject.toml
[project]
name = "app"
# src/a.py
# CODEOWNERS
* @org/app
```
