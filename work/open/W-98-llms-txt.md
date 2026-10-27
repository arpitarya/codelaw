---
type: Handoff
name: W-98
description: "Arpit, 2026-10-27 (W-36 gap G7): an llms.txt and a machine-readable rule catalogue generated from docs/rules/."
item: W-98
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-98 — `llms.txt` and a rule catalogue agents can fetch

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `scripts/rule-docs.py` also writes `llms.txt` (index) and `llms-full.txt` (every rule page) and a `rules.json` catalogue (id, pack, languages, fix kind, options schema, one-line do/don't) | agents and search can read codelaw's rules with no install | ⏳ |
| 2 | `--check` keeps them current in CI; linked from the README | they never drift | ⏳ |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Haiku.

Why: Prettier, Ruff and oxc serve `llms.txt`; none serves a per-rule schema'd catalogue ([proposal](../proposals/linter-for-ai-positioning.md) §3).

## Definition of done

- The three files are generated, deterministic, checked in CI, and list all rules with their fix kind.

## Record affected

SR-WORK-REGISTRY (the generated files) — no SR on behaviour.

## Hazards

- Size: `llms-full.txt` for 652 rules is large; keep `llms.txt` short.
