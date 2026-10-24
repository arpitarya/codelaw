---
type: Handoff
name: W-25
description: "Arpit, 2026-10-22: the repo review — records, the register and the work files that disagree with the tree."
item: W-25
filed: 2026-10-23
filed_by: "Arpit (Claude Code, Fable 5.1)"
updated: 2026-10-24
updated_by: "Claude (Opus 5.5), W-25/W-26 close-out 2026-10-24"
ball: agent
---

# W-25 — Bring the records and the work files back in line with the tree

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Give every component an owner: `crates/codelaw-graph` → SR-PROJECT-GRAPH; `fixtures/perf`, `perf/budgets.toml` → SR-LAW-8 (table rows and `owns:`); make `check-work-system.py` fail when a top-level entry of `crates/`, `scripts/`, `.claude/hooks/`, `fixtures/`, `packs/` has no row | the commit guard sees the graph crate and the budgets | ✅ 2026-10-24 |
| 2 | Set the register's `built` column from the tree: SR-LAW-7, SR-FORMATTER, SR-PROJECT-GRAPH, SR-AGENT-SURFACES, SR-CONSUMER-LAYOUT, SR-LAW-8, SR-BASELINE; make the check refuse `built: no` for a record whose `owns:` all exist | the register stops contradicting the records | ✅ 2026-10-24 |
| 3 | Rewrite the sentences that say nothing checks a law while the test exists (SR-LAW-3, SR-LAW-4 → `tests/laws.rs` `l3_*`/`l4_*`), and give SR-LINTER a veto that names a command that exists | law zero: a record's claim about its own enforcement is true | ✅ 2026-10-24 |
| 4 | Replace every backlog id that is no longer in `BACKLOG.md` (B-009, B-010, B-012, B-020, B-021, B-030, B-038) with the item or test that absorbed it; SR-EDITOR cites W-09, not W-08, for the extension | no pointer points at nothing (queue rule 45) | ✅ 2026-10-24 |
| 5 | One pass over `work/open/*.md`: strike blockers that landed (W-03 on W-07/08/09/12 and B-038; W-04 on W-07/08/09; W-21 "Blockers: None" and its stale `updated_by`; W-20 "the queue fails the check today"); add W-04's `**Model:**` line and move its quote under the table; `ball:` to `arpit` for 🔴 rows and `cowork` for W-05; make the check validate `ball:` against the row's colour | an item file and its row agree | ✅ 2026-10-24 |
| 6 | Drop the stored inbox `age` column; `inject-inbox.sh` and `/docket` compute it; the check fails a stored value | the 5-day rule runs on arithmetic, not a copied `0 d` | ✅ 2026-10-24 |
| 7 | Refresh SR-WORK-REGISTRY §3's "Last verified" dates in the same change, and name the 2026-10-20 `CLAUDE.md` edit (`8e2bbe8`) in a worklog line so SR-WORK-DOCS's veto is no longer tripped | the registry is current; the veto conditions hold | ✅ 2026-10-24 |

**Arpit, 2026-10-22:** *"review the repo, create work items to fix whatever was broken or whatever needs to be fixed or whatever needs to be structured in a better way"*

**Model:** Sonnet: every row is a text edit with a grep that proves it; the only judgment is which W-item absorbed each dead B-id, and the worklog says.

## Goal

`scripts/check-work-system.py` proves what the records claim — ownership complete, `built` true, every cited id live, every item file agreeing with its row — and `main` passes it.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · owners | the Ownership table names every top-level entry under the five folders; the check enforces it and passes | ✅ 2026-10-24 |
| 2 · built | no register row says `no` for a record whose owned components exist; the check enforces it | ✅ 2026-10-24 |
| 3 · claims | `grep -rn "Nothing checks this law yet" records/` is empty or every hit names a missing test that is a queue item | ✅ 2026-10-24 |
| 4 · ids | `grep -rhoE "B-[0-9]{3}" records work RELEASING.md README.md \| sort -u` ⊆ the ids in `work/BACKLOG.md` ∪ the ids named in `archive/README.md` | ✅ 2026-10-24 |
| 5 · files | `grep -L '^\*\*Model:\*\*' work/open/W-*.md` prints nothing; every `ball:` matches its row; no open file names a landed item as a blocker | ✅ 2026-10-24 |
| 6 · age | `OPEN-WORK.md`'s inbox table has no `age` column; `inject-inbox.sh` prints one computed from `filed` | ✅ 2026-10-24 |
| 7 · registry | every §3 row's "Last verified" is on or after its newest note | ✅ 2026-10-24 |

## Blockers

None. **Closed 2026-10-24** (W-25/W-26 close-out): every step and DoD holds; `check-work-system.py --self-test` and `docs-sync.py --check` pass.

## Record affected

[SR-WORK-OPEN-QUEUE](../../records/0051_WORK-open-queue.md) rules 39–40 (the `age` column, step 6) · [SR-WORK-REGISTRY](../../records/0060_WORK-registry.md) §3 and decisions 5–8 · [SR-WORK-OWNERSHIP](../../records/0058_WORK-ownership.md) decision 2 (completeness, step 1) · [SR-PROJECT-GRAPH](../../records/0103_project-graph.md), [SR-LAW-8](../../records/0010_LAW-8-performance-budgets.md) (`owns:`) · [SR-LAW-3](../../records/0005_LAW-3-languages.md), [SR-LAW-4](../../records/0006_LAW-4-every-rule-has-a-fix.md), [SR-LINTER](../../records/0106_linter.md), [SR-EDITOR](../../records/0114_editor.md) (one sentence each) · the register's `built` column.

## Hazards

- **Concurrent sessions edit the same work files** (`work/LESSONS.md`): re-derive `git status` before staging each step.
- **Step 1 widens the guard:** the first commit after it that touches `crates/codelaw-graph` must touch SR-PROJECT-GRAPH too.
- **Found by:** the 2026-10-22 review (`work/WORKLOG.md`, that date).

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- `records/README.md` §Ownership and the register table; `scripts/check-work-system.py`; `work/open/README.md` (the file contract).
