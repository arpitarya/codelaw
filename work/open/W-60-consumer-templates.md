---
type: Handoff
name: W-60
description: "Arpit, 2026-10-26: implement L10: one template per file codelaw writes into a consumer's repo, under templates/, with a registry, a slot-only renderer and a test; text moves out of init.rs, sync.rs, git_hooks.rs, rules_cmd.rs and agents/."
item: W-60
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-60 — Move every consumer-file text into `templates/` (L10)

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Inventory every consumer write (L10 decision 1): `init.rs` (`codelaw.toml`, `.codelaw/rules/README.md`, `.gitignore` line), `git_hooks.rs` + `src/hooks/*.sh`, `sync.rs` (instruction blocks, hook/MCP settings, Cowork plugin), `agents/` (skills, agents), `rules_cmd.rs` `new-rule`, CI workflows if W-51 has landed | the full list | ⏳ |
| 2 | Write the renderer: `{{name}}` slots, `\{{` escape, error on unfilled or unknown slot | no dependency | ⏳ |
| 3 | Move each text to `templates/<area>/<path>`; `agents/` moves to `templates/agents/` (L1: the old path's history stays in git; no archive copy needed for a move) | one place to edit | ⏳ |
| 4 | Write `templates/README.md`, the registry | what goes where, written by which command | ⏳ |
| 5 | The registry test (L10 decision 7) and an ownership row: `templates/` → SR-LAW-10 | the law is checked | ⏳ |
| 6 | Update CLAUDE.md §Layout, records citing `agents/`, and the register's `built` column for SR-LAW-10 | docs match the tree | ⏳ |

**Arpit, 2026-10-26:** *"create a law that all the files that are supposed to be created on the consumer end will have templates in separate templates directory. So that it can be edited easily. And changes are more visible."*

**Model:** Sonnet — a mechanical move with a written inventory and a test; outputs must stay byte-identical.

## Goal

- **Byte-identical output:** every command writes exactly what it wrote before the move — snapshot each consumer file before, compare after.
- No consumer text left in `crates/` except the merge mechanics (L10 decision 5).

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | for each writer, a fixture run before and after gives identical bytes | ⏳ |
| 2 · | `grep` finds no multi-line consumer text in `crates/*/src` (merge markers and JSON keys excepted) | ⏳ |
| 3 · | the registry test passes and fails when a template is added without a row | ⏳ |
| 4 · | `agents/` gone from the root; every link to it repointed | ⏳ |

## Blockers

None. A new writer added by another item (W-49, W-51) after this lands follows L10 from the start.

## Record affected

SR-LAW-10 (built) · SR-AGENT-SURFACES (paths of skills and agents) · SR-CLI-MCP (`init`, `new-rule`) · records/README.md §Ownership.

## Hazards

- `include_str!` paths move: the crates' relative paths to `templates/` must work from the packaged source too (`cargo package` / sdist).
- Concurrent sessions editing `agents/skills/*`: do the move in one commit and tell them in NOW.md.

## Reference

[SR-LAW-10](../../records/0012_LAW-10-consumer-templates.md)
