---
type: Handoff
name: W-19
description: "Arpit, 2026-10-20: L8's budgets, which are all unset because nothing has been measured."
item: W-19
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-19 — Build `codelaw bench`, measure, and propose the L8 budgets

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Build `codelaw bench` and the reference workload | a repeatable measurement | ⏳ |
| 2 | Measure the SR-LINTER and SR-ARCHITECTURE targets (warm/cold 10k files, one-file hook, fix, LSP edit, startup, peak memory) | numbers to set budgets from | ⏳ |
| 3 | Propose `perf/budgets.toml` values | the decider sets the budgets | ⏳ |
| 4 | Add the CI perf gate | a budget is never exceeded | ⏳ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: a workload that represents real repos and a stable measurement on shared runners are judgment calls.

## Goal

`perf/budgets.toml` holds measured budgets the decider set, and CI fails a change that exceeds one.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · bench | `codelaw bench` runs the reference workload and prints each budgeted operation. | ⏳ |
| 2 · measure | Every target in SR-ARCHITECTURE decision 29 and SR-LINTER decision 8 is measured on named hardware. | ⏳ |
| 3 · propose | A budget proposal goes to the decider (inbox row). | ⏳ |
| 4 · gate | CI compares each run against `perf/budgets.toml` ([L8](../../records/0010_LAW-8-performance-budgets.md) decision 7). | ⏳ |

## Blockers

None to measure; setting the budgets is the decider's.

## Record affected

[SR-LAW-8](../../records/0010_LAW-8-performance-budgets.md) · [SR-ARCHITECTURE](../../records/0108_architecture.md) decision 29 · [SR-LINTER](../../records/0106_linter.md) decision 8.

## Hazards

- **A loaded machine produces a clean, localised anomaly** (LESSONS): say what else is running when measuring.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
