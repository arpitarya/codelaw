---
type: Handoff
name: W-96
description: "Arpit, 2026-10-27 (W-36 gap G4): pick --format agent automatically when codelaw runs inside a coding agent."
item: W-96
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-96 — Choose `--format agent` automatically inside a coding agent

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Detect the agent from its documented environment variables (Claude Code `CLAUDECODE`, Codex `CODEX_*`, Cursor `CURSOR_AGENT`, Gemini CLI `GEMINI_CLI`, and the others oxlint's `agent_detection.rs` lists — each checked against that agent's docs) | the agent reads the short format with no flag | ⏳ |
| 2 | An explicit `--format` always wins; `CODELAW_FORMAT` overrides detection; `codelaw config --print` shows which applied and why | no surprise | ⏳ |
| 3 | Tests per variable; docs in USAGE | it stays true | ⏳ |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Haiku.

Why: oxlint auto-selects its agent format since 1.66.0 (2026-05-18); codelaw needs the flag ([proposal](../proposals/linter-for-ai-positioning.md) §2).

## Definition of done

- With `CLAUDECODE=1` and no `--format`, `codelaw check` prints the agent format; with `--format json` it prints JSON; with neither variable it prints text.

## Record affected

SR-CLI-MCP decision 4 (the formats) — the environment becomes a stated input of the default.

## Hazards

- L5: the output still depends only on inputs; the environment variable is one, and the record must say so.
