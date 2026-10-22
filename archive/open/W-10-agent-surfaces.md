---
type: Handoff
name: W-10
description: "Arpit, 2026-10-20: the agent-facing set SR-AGENT-SURFACES specifies."
item: W-10
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-22
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-10 — Build the agent surfaces — hook mode, `sync-agents`, the skills, `init --agents`

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Add `codelaw hook` (one file, agent output, the hook budget) | agents get findings after every edit | ✅ 2026-10-22 |
| 2 | Add `codelaw sync-agents [--check]` writing each agent's files with marker blocks | one source, every agent | ✅ 2026-10-22 |
| 3 | Write the skills SR-AGENT-SURFACES catalogues, `codelaw-fix` first | L4 decision 10: a skill carries every manual fix | ✅ 2026-10-22 |
| 4 | `init --agents claude,cowork,copilot,codex,kiro` | set up in one command | ✅ 2026-10-22 |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: the skills are prose an agent follows; their quality is the deliverable and no test grades it.

## Goal

Hook mode, `sync-agents` and the skills ship in both packages; `init --agents` writes exactly SR-CONSUMER-LAYOUT's agent paths.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · hook | `codelaw hook` checks one file and prints agent output (SR-AGENT-SURFACES). | ✅ `crates/codelaw-agents/src/hook.rs`; decision 6. Tests: `hook_fixes_the_edited_file_and_returns_what_is_left` (payload on stdin, exit 2, stderr, same bytes as `fix`), and `mcp_parity.rs` now runs `hook` on every fixture file and compares the bytes with `fix`'s (L5 decision 5). |
| 2 · sync | `sync-agents --check` fails on drift; text outside the markers is untouched. | ✅ `crates/codelaw-agents/src/sync.rs`; decision 7. Test: `init_agents_writes_the_layout_and_sync_keeps_it` (team edits outside the markers pass `--check`; an edit inside them or to a generated skill exits 1; `sync-agents` restores the block and keeps the team's text; stale `codelaw-*` skills removed, the team's kept). |
| 3 · skills | Every skill in the catalogue exists; `fix_skill:` names resolve at load (L4 decision 10). | ✅ the 12 skills, the custom agent and the trigger evals in `agents/`, compiled into the binary; `skills.rs` tests the Agent Skills format, the safety section, the evals, and that the catalogue equals `codelaw_rules::KNOWN_SKILLS`. |
| 4 · init | `init --agents` writes only the paths SR-CONSUMER-LAYOUT lists (W-20 tests it). | ✅ `init --agents` writes `[agents] targets` and the files after the packs; the same test checks every new path against SR-CONSUMER-LAYOUT §1 (which now names `.agents/` and `.vscode/mcp.json`), and that with no `--agents` no agent file exists. |

## Blockers

None. W-08 has landed; `.mcp.json` and the skills point at `codelaw mcp`.

## Record affected

[SR-AGENT-SURFACES](../../records/0102_agent-surfaces.md) decisions 6–7 (new) · [SR-CONSUMER-LAYOUT](../../records/0111_consumer-layout.md) §1 · [SR-CONFIG](../../records/0110_config.md) decision 8 · [SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decision 6 · [SR-PACKAGES](../../records/0107_packages.md) decision 2 · [SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 6 · [SR-LAW-4](../../records/0006_LAW-4-every-rule-has-a-fix.md) decision 10.

## Left for later (not in this DoD)

- SR-AGENT-SURFACES decision 5's **task evals** and running the evals in CI against each agent: only the trigger evals exist.
- The Kiro Power bundle (decision 2, optional) is not generated.
- The Cowork plugin path and the Kiro v1 hook schema are the least verified rows of decision 7.

## Hazards

- **[W-05](../../work/open/W-05-write-time-guidance.md)** (research) may add `hook --pre` and `brief`; the hook is built so a pre-write mode joins it as a flag (decision 6).

## Reference

- [`work/OPEN-WORK.md`](../../work/OPEN-WORK.md): this item's row.
