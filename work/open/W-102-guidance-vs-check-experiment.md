---
type: Handoff
name: W-102
description: "Arpit, 2026-10-27 (W-36 R4): measure whether an agent with Fux and the rule docs still breaks rules — the evidence for or against the README's 'you need a check' line."
item: W-102
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-102 — Experiment — does good guidance make the check unnecessary?

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | 20 small tasks in a fixture repo with `js/recommended`, `py/recommended`, three custom rules and two configured `use-instead` rules | a fair task set | ⏳ |
| 2 | Three arms, same model and prompt, 3 runs each (180 runs): (A) AGENTS.md only; (B) + codelaw MCP and the rule pages; (C) + a Fux index | comparable runs | ⏳ |
| 3 | Measure `codelaw check` on each diff (new findings by rule), with `hook` and `hook --pre` off; record tokens and turns | the data | ⏳ |
| 4 | Decide: arm C gives zero new `error`s in all 60 runs → the README's "rules it can't skip" line goes; otherwise it stays with the measured rate | a README line backed by data | ⏳ |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Claude Code (Opus) on the Mac.

Design: [proposal](../proposals/linter-for-ai-positioning.md) §5. Needs a real agent — a Claude Code session on the Mac.

## Definition of done

- The 180 runs, their inputs and the per-arm counts written up; the README line kept or removed per step 4.

## Record affected

No SR affected — an experiment. README by W-35's rule.

## Hazards

- Agent runs cost tokens and time; budget them before starting (L9's disk budget applies to the fixture repos).
- Non-determinism: report rates with the run count, never a single run.
