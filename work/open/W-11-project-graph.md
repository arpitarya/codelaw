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
| 1 | Build `codelaw-graph`: link each file's imports and exports (from the semantic model) with its own resolver (not `oxc_resolver`: SR-PROJECT-GRAPH decision 2) | a whole-repo symbol graph | ✅ `crates/codelaw-graph`; 25 tests |
| 2 | Run `scope: project` rules (`graph:` blocks) | `graph/no-unused-exports` in js and python, `graph/no-unused-css-classes` in css | ✅ run by `codelaw check` on the corpus |
| 3 | Add `graph refs` and `graph why-unused` | the debug commands | ✅ used to classify the corpus findings |
| 4 | Measure false positives on pinned real repos | the veto check (was B-013) | ✅ 14 of 145 false (9.7%), none beyond config: veto not met |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: module resolution, entry points and dynamic imports decide false positives, and the corpus measurement is judgment.

## Goal

`graph/no-unused-exports` runs on monorepos with the record's entry-point and keep-tag rules, and its false-positive rate is measured.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · graph | Imports/exports from `codelaw-semantic` are linked per project root (SR-ARCHITECTURE decision 17). | ✅ `facts.rs` reads `codelaw_semantic::build`; `link.rs` links; `crates/codelaw-graph` tests |
| 2 · rules | `graph:` rules load and run; the pack rules waiting on the graph move out of the packs' "not ported yet" lists. | ✅ three `graph/` rules and the six `node/` import rules in `packs/`; none waits on W-11 in a README |
| 3 · commands | `codelaw graph refs <symbol>`, `graph why-unused <symbol>`. | ✅ `crates/codelaw-cli/src/graph_cmd.rs`; test `why_unused_and_refs` |
| 4 · corpus | A false-positive rate on pinned repos is recorded in SR-PROJECT-GRAPH. | ✅ Consequences: seven repos at pinned commits, every finding classified |

**2026-10-22:** the six `node/` import rules run on the graph through a new `graph: import:` query, and files outside `[files]`/`languages` count for uses but are not reported. Unverified when committed (the disk was full); verified after the merge.

**2026-10-22, corpus run:** seven repos (flask, httpx, starlette, zustand, excalidraw, full-stack-fastapi-template, djangoproject.com) at pinned commits; 145 findings, 14 false (9.7%), each expressible by `[graph]` config, so the veto condition does not hold. Fixed on the way: unittest `test*.py` and Django template-tag decorators as defaults, module augmentation as a use, CSS linked from HTML still checked, and the two pack copies of `graph/no-unused-exports` made identical. Left as judgment calls for the decider, not fixed: a `private` root package published from a build folder (zustand: config `entry`), and stylesheets nothing in the repo links, styling generated pages (MkDocs `extra_css`: config `keep`).

## Blockers

None.

## Record affected

[SR-PROJECT-GRAPH](../../records/0103_project-graph.md) · [SR-RULE-FORMAT](../../records/0112_rule-format.md) decision 3 · [SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 2.

## Hazards

- **`oxc_resolver`** is not used: the graph has its own resolver (SR-PROJECT-GRAPH decision 2).
- **L8:** linking runs after every file; measure it in W-19.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
