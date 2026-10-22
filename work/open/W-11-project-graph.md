---
type: Handoff
name: W-11
description: "Arpit, 2026-10-20: the cross-file graph SR-PROJECT-GRAPH specifies, `graph/no-unused-exports` first."
item: W-11
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-22
updated_by: "Claude Code (Opus 5.5)"
ball: agent
---

# W-11 — Build the project graph and `graph:` rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Build `codelaw-graph`: link each file's imports and exports (from the semantic model) with `oxc_resolver` | a whole-repo symbol graph | ⏳ |
| 2 | Run `scope: project` rules (`graph:` blocks) | `graph/no-unused-exports` in js and python, `graph/no-unused-css-classes` in css | ⏳ |
| 3 | Add `graph refs` and `graph why-unused` | the debug commands | ⏳ |
| 4 | Measure false positives on pinned real repos | the veto check (was B-013) | 🟡 first pass run (2026-10-22): Flask registrars added; rate not yet judged |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: module resolution, entry points and dynamic imports decide false positives, and the corpus measurement is judgment.

## Goal

`graph/no-unused-exports` runs on monorepos with the record's entry-point and keep-tag rules, and its false-positive rate is measured.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · graph | Imports/exports from `codelaw-semantic` are linked per project root (SR-ARCHITECTURE decision 17). | ⏳ |
| 2 · rules | `graph:` rules load and run; the pack rules waiting on the graph move out of the packs' "not ported yet" lists. | ⏳ |
| 3 · commands | `codelaw graph refs <symbol>`, `graph why-unused <symbol>`. | ⏳ |
| 4 · corpus | A false-positive rate on pinned repos is recorded in SR-PROJECT-GRAPH. | ⏳ |

**2026-10-22:** the six `node/` import rules run on the graph through a new `graph: import:` query, and files outside `[files]`/`languages` count for uses but are not reported. Unverified when committed (the disk was full); verified after the merge.

## Blockers

None.

## Record affected

[SR-PROJECT-GRAPH](../../records/0103_project-graph.md) · [SR-RULE-FORMAT](../../records/0112_rule-format.md) decision 3 · [SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 2.

## Hazards

- **`oxc_resolver`** needs a record naming it as a dependency (L2 permits, a record decides).
- **L8:** linking runs after every file; measure it in W-19.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
