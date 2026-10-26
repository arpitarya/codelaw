---
type: Handoff
name: W-46
description: "Arpit, 2026-10-25: a report of every exception (baseline, deprecation, inline ignore, dated override) with --expiring 30d and grouping by added_by, rule or file."
item: W-46
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-46 — `codelaw exceptions` — every exception, what expires, who owns it

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Define the command in SR-CLI-MCP and the exception list in SR-BASELINE | the record is the spec | ✅ |
| 2 | Collect every exception: baseline entries, deprecations, inline `codelaw-ignore`, `[[overrides]]` with `expires` | one list | ✅ |
| 3 | Filters: `--expiring 30d`, `--expired`, `--mine` (git `user.email`), `--rule`, `--path` | the slice you care about | ✅ |
| 4 | `--group-by added_by|rule|file|expires-month`; `--format text|json|csv|markdown` | a table for a stand-up or a PR comment | ✅ |
| 5 | MCP twin `list_exceptions` | agents see what's about to expire | ✅ |

**Arpit, 2026-10-25:** *"Watch mode, let's build it. Then CI report formats, let's build it. Exception report, that is code law exceptions, expiring 30 days, group added by … build it. Baseline burn down over commits, build it. Code law migrate from ESLint, flat config, Ruff, style lint, as well as biome … Remove OXLint. Dev server plugin for RSpack, Vite, Webpack, Next.js. Pre-commit hook, post-commit hook, create work items for all these."*

**Model:** Sonnet — a read-only report over data W-40 defines.

## Goal

```
$ codelaw exceptions --expiring 30d --group-by added_by
as-of 2026-10-25 (HEAD commit date)
arpit@example.com  3 exceptions, 7 uses
  2026-11-02  in 8 days   js/no-console        src/debug/panel.ts (2)  debug panel, ORD-1600
  …
(no added_by)      1 exception
```
- Days left use the as-of date (SR-BASELINE decision 4), never the wall clock.
- Exit 0 always; `--fail-if-expiring 7d` exits 1, for a scheduled job.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | every exception form appears once, with expires, days left, added_by, reason, count | ✅ |
| 2 · | `--group-by` and `--expiring` combine | ✅ |
| 3 · | csv and markdown render in a GitHub comment | ✅ |
| 4 · | `--as-of` changes the days-left column and nothing else | ✅ |

## Blockers

none: W-40 landed. Built 2026-10-26; the listing is SR-BASELINE decision 11. `[[overrides]]` carry no `expires` (decision 9 calls them configuration), so they are not listed.

## Record affected

SR-CLI-MCP (the command and the MCP tool) · SR-BASELINE (what counts as an exception — W-40's decision 9).

## Hazards

- Inline ignores need a full-tree scan; reuse the cache's parsed comments, don't re-read files.

## Reference

[W-40](W-40-dated-exceptions.md)
