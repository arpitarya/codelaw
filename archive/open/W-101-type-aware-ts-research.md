---
type: Handoff
name: W-101
description: "Arpit, 2026-10-27 (W-36 R3): reopen the parked type-aware-rules proposal (B-049) as research, now that oxlint's type-aware linting is stable."
item: W-101
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: cowork
closed: 2026-10-27
---

# W-101 — Research — type-aware TypeScript rules, reopened

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Re-read [the parked proposal](../../work/proposals/type-aware-rules.md) against today: oxlint type-aware stable 2026-07-22 (59/61 typescript-eslint rules via tsgolint/tsgo), Biome's own inference, TypeScript 7 (tsgo) and its API status | is the typescript-go route still right? — no: TS 7.1's own API | ✅ |
| 2 | What it costs codelaw: L3 and L6 amendments, a helper binary per platform, L5 (no network) and L8 (budgets) | the price, named — [proposal](../../work/proposals/type-aware-rules.md) §5 | ✅ |
| 3 | A proposal with a pick for the decider | a decision — ruled R1–R8, 2026-10-27 | ✅ |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../../work/proposals/linter-for-ai-positioning.md).

**Model:** Cowork, Opus — research (rule 37a).

Graduates [B-049](../../work/BACKLOG.md) (row deleted in this change).

## Definition of done

- A proposal names a route, its cost and its trigger; an inbox row for the pick.

## Record affected

No SR affected — research. Its outcome may amend L3, L6 and SR-ARCHITECTURE, each by its own item.

## Hazards

- oxlint wraps a Go helper; codelaw's L2/L11 apply to anything it would ship.

## Outcome (2026-10-27)

Ruled by voice, R1–R8: TypeScript 7.1's own API through the project's installed copy, a Node helper, rules in the `ts` pack, preview on the 7.1 RC, oxlint as the gap filler, a version watch. Built by W-104–W-107. [Proposal](../../work/proposals/type-aware-rules.md).
