---
type: Handoff
name: W-13
description: "Arpit, 2026-10-20: SR-BASELINE's grandfathering and the time and change-set inputs L5 names."
item: W-13
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-13 — Build baselines, the deprecations registry, `--as-of` and the git inputs

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read `HEAD`'s commit date as the as-of default; add `--as-of` | L5: time is an explicit input | ⏳ |
| 2 | Add `--changed-since` and `--staged` through the `git` binary | SR-ARCHITECTURE decision 14 | ⏳ |
| 3 | Build count-based baselines with `until`, `on_touch` and the ratchet | grandfather old violations | ⏳ |
| 4 | Add the deprecations registry and `graph/no-deprecated-usage` | the engine's own rule | ⏳ |
| 5 | Check whether count-based baselines let a new use offset a removed one | the veto check (was B-014) | ⏳ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: deadlines, touch policy and ratchet interact, and the 'offset' hole is a judgment the tests can only sample.

## Goal

Baselines and deprecations work as SR-BASELINE specifies, with time and the change set taken only from explicit inputs.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · time | The as-of date is `HEAD`'s commit date or `--as-of`; never the wall clock (L5). | ⏳ |
| 2 · git | `--changed-since <ref>`, `--staged`; without `git` they say so and the rest runs. | ⏳ |
| 3 · baseline | `.codelaw/baseline/<rule>.toml`, count-based, `until`, `on_touch`, ratchet, CODEOWNERS. | ⏳ |
| 4 · deprecations | `.codelaw/deprecations.toml` and `graph/no-deprecated-usage` (uses W-11's graph). | ⏳ |
| 5 · veto | The offset check is run and its result recorded in SR-BASELINE. | ⏳ |

## Blockers

Waiting on W-11 for `graph/no-deprecated-usage`; the rest can start.

## Record affected

[SR-BASELINE](../../records/0104_baseline-and-deprecations.md) · [SR-LAW-5](../../records/0007_LAW-5-no-model-in-the-engine.md) · [SR-ARCHITECTURE](../../records/0108_architecture.md) decision 14.

## Hazards

- **The wall clock** must never leak in: a test runs the same repo on two dates and expects the same output.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
