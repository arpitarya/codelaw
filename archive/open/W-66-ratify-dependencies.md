---
type: Handoff
name: W-66
description: "Arpit, 2026-10-26: the decider rules each of the 32 pending dependency units in DEPENDENCIES.md — ratify or remove — one by one."
item: W-66
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: done
---

# W-66 — ratify or remove each pending dependency

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Walk `DEPENDENCIES.md` one unit at a time (docket style): what it does, what replacing it would cost, a recommendation | an informed ruling per unit | ✅ 2026-10-26, by voice: 32 cards |
| 2 | Write each ruling into its row (`ratified YYYY-MM-DD` with his words, or `remove`) | the list is the truth | ✅ 29 ratified, 3 remove |
| 3 | Each `remove` opens its own work item: codelaw's replacement first (L7 keeps every formatter), then the dependency goes | no gap in what codelaw does | ✅ one item for all three: W-67 |

**Arpit, 2026-10-26:** *"tell me what all dependencies do we have. We will ratify them one by one."*

**Model:** NONE for the rulings (the decider's); Opus for each card's recommendation — what a replacement costs is judgment.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | no `pending` row left in `DEPENDENCIES.md` | ✅ |
| 2 · | every `remove` has a work item | ✅ W-67 |

## Rulings

**Arpit, 2026-10-26 (voice):** *"Ratify."* for D-01–D-19, D-21, D-22, D-25–D-32; strsim (D-23) and anyhow (D-24) — *"Remove"*; postcard (D-20) — *"Replace."*

## Blockers

None — closed 2026-10-26.

## Record affected

SR-LAW-11 (its list); each `remove` names its own records.

## Hazards

- Removing a formatter (D-01, D-02, D-06–D-09) means writing one: years of edge cases. Rule those last, with the cost on the card.

## Reference

[`DEPENDENCIES.md`](../DEPENDENCIES.md)
