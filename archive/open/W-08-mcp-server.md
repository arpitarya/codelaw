---
type: Handoff
name: W-08
description: "Arpit, 2026-10-20: the MCP server SR-CLI-MCP and L6 decision 3 require."
item: W-08
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-08 — Build the MCP server — `codelaw mcp`

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Add `codelaw-mcp` and `codelaw mcp` (stdio) with the tools SR-CLI-MCP decision 5 lists | agents query codelaw directly | ✅ |
| 2 | Every tool calls the same engine functions as the CLI | one engine, two doors | ✅ |
| 3 | A parity test: CLI and MCP give the same answer on every fixture | SR-CLI-MCP decision 6 checked (was B-025) | ✅ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Sonnet: the tools and their JSON shapes are specified in SR-CLI-MCP, and a parity test catches a wrong answer.

## Goal

`codelaw mcp` serves SR-CLI-MCP's tools over stdio, with answers byte-equal to the CLI's JSON.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · server | `codelaw mcp` speaks MCP over stdio; every tool of SR-CLI-MCP decision 5 is listed and works. | ✅ |
| 2 · parity | One test runs every fixture through CLI and MCP and compares the JSON. | ✅ |
| 3 · gate | `scripts/law-gate.py` no longer lists `mcp`. | ✅ |

**Done 2026-10-20.** Two tools stand on work that is not built yet: `format_snippet` returns what `fix` writes, and gains the format step when W-07 gives `fix` one; `trace_rule` is `rules --why --for`, and `match --explain`'s matcher trace (W-16) is to return through it. The warm, cached rule set and the socket are W-15's.

## Blockers

None.

## Record affected

[SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decisions 5–6 · [SR-PACKAGES](../../records/0107_packages.md) decision 1.

## Hazards

- **No model, no network** ([L5](../../records/0007_LAW-5-no-model-in-the-engine.md)): the server answers from the engine only.
- **An MCP SDK dependency** must pass L2; a hand-written JSON-RPC loop is the fallback.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
