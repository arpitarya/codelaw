---
type: Handoff
name: W-10
description: "Arpit, 2026-10-20: the agent-facing set SR-AGENT-SURFACES specifies."
item: W-10
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-10 — Build the agent surfaces — hook mode, `sync-agents`, the skills, `init --agents`

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Add `codelaw hook` (one file, agent output, the hook budget) | agents get findings after every edit | ⏳ |
| 2 | Add `codelaw sync-agents [--check]` writing each agent's files with marker blocks | one source, every agent | ⏳ |
| 3 | Write the skills SR-AGENT-SURFACES catalogues, `codelaw-fix` first | L4 decision 10: a skill carries every manual fix | ⏳ |
| 4 | `init --agents claude,cowork,copilot,codex,kiro` | set up in one command | ⏳ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: the skills are prose an agent follows; their quality is the deliverable and no test grades it.

## Goal

Hook mode, `sync-agents` and the skills ship in both packages; `init --agents` writes exactly SR-CONSUMER-LAYOUT's agent paths.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · hook | `codelaw hook` checks one file and prints agent output (SR-AGENT-SURFACES). | ⏳ |
| 2 · sync | `sync-agents --check` fails on drift; text outside the markers is untouched. | ⏳ |
| 3 · skills | Every skill in the catalogue exists; `fix_skill:` names resolve at load (L4 decision 10). | ⏳ |
| 4 · init | `init --agents` writes only the paths SR-CONSUMER-LAYOUT lists (W-20 tests it). | ⏳ |

## Blockers

Waiting on W-08: `.mcp.json` and the skills point at `codelaw mcp`.

## Record affected

[SR-AGENT-SURFACES](../../records/0102_agent-surfaces.md) · [SR-CONSUMER-LAYOUT](../../records/0111_consumer-layout.md) · [SR-LAW-4](../../records/0006_LAW-4-every-rule-has-a-fix.md) decision 10.

## Hazards

- **[W-05](W-05-write-time-guidance.md)** (research) may add `hook --pre` and `brief`; build the hook so a pre-write mode can join it.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
