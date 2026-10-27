---
type: Handoff
name: W-100
description: "Arpit, 2026-10-27 (W-36 gap G8): promote rules from preview to stable by pinned corpus runs."
item: W-100
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-100 — Promote rules out of `preview` by corpus

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The promotion criterion in a record: a rule is `stable` after N pinned corpus repos with zero false positives against its source tool's verdict, and its fix verified | "every rule is preview" stops being an objection | ⏳ |
| 2 | The corpus CI job reports per-rule agreement; a script proposes promotions; the change lands as a diff to `stability:` | promotion is evidence, not opinion | ⏳ |
| 3 | First batch promoted; README FAQ updated | users see stable rules | ⏳ |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Sonnet.

Why: a platform team's first objection is "every rule is `preview`" (README FAQ) ([proposal](../proposals/linter-for-ai-positioning.md) §6).

## Definition of done

- At least one pack's first batch is `stable` with the corpus run that proved it named in each rule's history.

## Record affected

SR-RULE-FORMAT (`stability`) · SR-RULE-PACKS.

## Hazards

- Comparing with the source tool needs it installed in CI — L11, as W-99.
