---
type: Handoff
name: W-99
description: "Arpit, 2026-10-27 (W-36 gap G5): measure codelaw against ESLint, Prettier, Biome, oxlint and Ruff on the same pinned repos and runner, before any speed claim."
item: W-99
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-99 — A side-by-side benchmark before any speed claim

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Name each rival at an exact version as a CI-only bench tool, one row each in L11's list (§3 of `records/0013_LAW-11-dependencies.md`) (L11 — the decider ratifies each row by name when it lands) | the tools may be installed in CI | ⏳ then 🔴 Arpit ratifies the rows |
| 2 | A `bench-rivals` CI job (manual and weekly): the three pinned corpus repos, the same runner image, same day; equivalent rule sets (recommended presets), lint and format separately, cold and warm | numbers a reader can reproduce | ⏳ |
| 3 | Results in L8 §Measurements as context rows; the README states a claim only from these, with the date and machine | an honest claim | ⏳ |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Sonnet.

Why: every rival publishes a speed claim (Biome "~35x Prettier", oxlint "50–100x ESLint", Ruff "10–100x Flake8"); codelaw has none, and makes none until measured ([proposal](../proposals/linter-for-ai-positioning.md) §4).

## Definition of done

- One CI run produces a table per repo: tool, version, rule set, cold and warm p50/p95; the README cites it or says nothing.

## Record affected

SR-LAW-11 §3 (CI-only rows) · SR-LAW-8 §Measurements (context rows).

## Hazards

- Equal work is hard: compare like rule sets, say where they differ.
- A loaded machine produces a clean anomaly (LESSONS) — runner only.
