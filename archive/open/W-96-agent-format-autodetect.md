---
type: Handoff
name: W-96
description: "Arpit, 2026-10-27 (W-36 gap G4): pick --format agent automatically when codelaw runs inside a coding agent."
item: W-96
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5), W-96 close"
ball: done
---

# W-96 — Choose `--format agent` automatically inside a coding agent

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Detect the agent from its documented environment variables (Claude Code `CLAUDECODE`, Codex `CODEX_*`, Cursor `CURSOR_AGENT`, Gemini CLI `GEMINI_CLI`, and the others oxlint's `agent_detection.rs` lists — each checked against that agent's docs) | the agent reads the short format with no flag | ✅ `CLAUDE_CODE_CHILD_SESSION`, `CLAUDECODE` (not on a terminal: IDE extensions set it there), `CODEX_THREAD_ID`, `CODEX_SANDBOX`, `CODEX_SANDBOX_NETWORK_DISABLED`, `GEMINI_CLI` — `crates/codelaw-report/src/agent_env.rs` |
| 2 | An explicit `--format` always wins; `CODELAW_FORMAT` overrides detection; `codelaw config --print` shows which applied and why | no surprise | ✅ `resolve_format`; `config --print` value `format` with its source |
| 3 | Tests per variable; docs in USAGE | it stays true | ✅ `crates/codelaw-cli/tests/agent_format.rs` (3), `agent_env` unit tests (3); USAGE §After; SR-CLI-MCP decision 4a |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../../work/proposals/linter-for-ai-positioning.md).

**Model:** Haiku.

Why: oxlint auto-selects its agent format since 1.66.0 (2026-05-18); codelaw needs the flag ([proposal](../../work/proposals/linter-for-ai-positioning.md) §2).

## Definition of done

- ✅ With `CLAUDECODE=1` and no `--format`, `codelaw check` prints the agent format; with `--format json` it prints JSON; with neither variable it prints text. — `agent_format.rs::an_agent_gets_the_agent_format_and_a_flag_wins`.

## Closed (2026-10-27)

- **Not detected, because no agent's own docs or source confirmed them** (checked 2026-10-27): `CURSOR_AGENT` (Cursor's CLI docs do not name it), `COPILOT_CLI` (Copilot CLI's docs name only `COPILOT_HOME`), `OPENCODE`, `JUNIE_DATA`/`JUNIE_SHIM_PATH`, `AI_AGENT`, `CLAUDE_CODE` — all named only by oxlint's `agent_detection.rs`. Adding one is a row of `AGENT_VARS` and of SR-CLI-MCP decision 4a's table once its agent documents it.
- **Left out on purpose:** oxlint's `TERM_PROGRAM`∋`kiro`, `EDITOR`∋`devin`, `PATH`∋`.pi/agent` and `REPL_ID`: each also holds in a terminal a person types in.
- **CI and a terminal:** `CI`, `GITHUB_ACTIONS`, `GITLAB_CI`, `TERM_PROGRAM` are not agent variables (tested); `CLAUDECODE` on a terminal is not either, since Claude Code's IDE extensions set it in the integrated terminal.
- This repo's `.cargo/config.toml` sets `CODELAW_FORMAT=text` so the test suite, run inside an agent, still sees `text`.

## Record affected

SR-CLI-MCP decision 4 (the formats) — the environment becomes a stated input of the default.

## Hazards

- L5: the output still depends only on inputs; the environment variable is one, and the record must say so.
