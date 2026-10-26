---
type: Handoff
name: W-47
description: "Arpit, 2026-10-25: how the baselined violations shrank over time, per rule, read from git history without re-linting."
item: W-47
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5), parallel merge"
ball: done
---

# W-47 — `codelaw baseline history` — the burn-down over commits

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Define `baseline history` in SR-BASELINE | the record is the spec | ✅ |
| 2 | Walk the commits that touched `.codelaw/baseline/` (`git log`), read each version with `git show`, sum counts per rule | a series per rule, fast | ✅ |
| 3 | Options: `--since <ref|date>`, `--rule`, `--by day|week|commit` | the window you want | ✅ |
| 4 | Output: text table with a sparkline; `json` and `csv` for dashboards; `markdown` for PR comments | shareable | ✅ |
| 5 | Docs: a weekly CI job that posts it | the trend lands where people look | ✅ |

**Arpit, 2026-10-25:** *"Watch mode, let's build it. Then CI report formats, let's build it. Exception report, that is code law exceptions, expiring 30 days, group added by … build it. Baseline burn down over commits, build it. Code law migrate from ESLint, flat config, Ruff, style lint, as well as biome … Remove OXLint. Dev server plugin for RSpack, Vite, Webpack, Next.js. Pre-commit hook, post-commit hook, create work items for all these."*

**Model:** Sonnet — git plumbing and a sum; fixed output.

## Goal

```
$ codelaw baseline history --since 90d --by week
rule                       start  now  change  trend
graph/no-deprecated-usage    212   97   −115   █▇▆▅▄▃▃▂
react/no-context              41   38     −3   ████▇▇▇▇
total                        253  135   −118
```
- Counts come from the committed baseline files only, so the series is deterministic and cheap.
- `--relint` (later, not this item) would re-run `check` per commit for rules without a baseline.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | history on a fixture repo with five baseline commits matches hand-counted totals | ✅ |
| 2 · | a renamed or deleted baseline file is handled | ✅ |
| 3 · | same commit range → byte-identical output | ✅ |
| 4 · | no `git` → usage error, exit 2 | ✅ |

## Blockers

None. Landed on the branch; the parent merge writes the queue rows.

## Record affected

SR-BASELINE (a new decision) · SR-CLI-MCP (the command).

## Hazards

- Shallow CI clones have no history: say so and suggest `fetch-depth: 0`.

## Reference

[Notion's ratchet](https://www.notion.com/blog/how-we-evolved-our-code-notions-ratcheting-system-using-custom-eslint-rules) · [Betterer](https://phenomnomnominal.github.io/betterer/)
