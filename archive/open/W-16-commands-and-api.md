---
type: Handoff
name: W-16
description: "Arpit, 2026-10-20: the CLI surface SR-CLI-MCP lists that isn't built, and the API SR-PACKAGES decision 3 promises."
item: W-16
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-22
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-16 — Build the missing commands, flags, the config schema and the full JS/Python API

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Add `config --print [--for]`, `config --schema` and ship `codelaw.schema.json` | every value traceable; editors validate | ✅ |
| 2 | Add `match --explain`, `fix --trace`, `--timings`, `--format sarif` | the debug and CI surfaces | ✅ |
| 3 | Add `rules --for <file> --format json`, then `rulesFor`/`rules_for` and `format` in the npm and Python APIs | the API SR-PACKAGES names | ✅ |
| 4 | Test that every config key is defined in exactly one record and present in the schema | SR-CONFIG's veto (was B-022) | ✅ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Sonnet: each command and flag is specified, and output can be tested against fixtures.

## Goal

Every command and flag SR-CLI-MCP lists exists, except those owned by other items (`graph`, `bench`, git inputs), and both APIs match SR-PACKAGES decision 3.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · config | `config --print` shows each value's source; `codelaw.schema.json` ships in npm `codelaw` and via `config --schema`. | ✅ `tests/debug.rs` `config_print_…`; `scripts/package.py` ships and checks the schema |
| 2 · debug | `match --explain`, `fix --trace`, `--timings`. | ✅ `fix --trace` traces the engine's own loop (one run, `fix_loop_with`); MCP `trace_rule` carries `for.match` |
| 3 · SARIF | `--format sarif` validates against the SARIF 2.1.0 schema. | ✅ `tests/debug.rs` `sarif_is_valid_sarif_2_1_0` (`boon`) |
| 4 · API | `rulesFor`/`rules_for` and `format` (after W-07) in both APIs, same JSON. | ✅ `format --format json`; `tests/api.rs` runs node and python3 |
| 5 · schema test | Config keys ↔ records ↔ schema, one test. | ✅ `crates/codelaw-config/tests/schema.rs` |

## Blockers

None. W-07 landed; the API's `format` is built on it.

**2026-10-22:** the WIP merged on 2026-10-22 was reconciled with `main`: one rules-listing path (`query.rs`'s `list_rules`/`rules_for`, JSON `for` + `rules[]`), the duplicates `rule_cards`, `RuleCard`, `rule_cards_json/text` and `trace_fix` removed, and `fix --trace` recording the very loop `fix` keeps.

## Record affected

[SR-CLI-MCP](../../records/0115_cli-and-mcp.md) · [SR-CONFIG](../../records/0110_config.md) decisions 6–7 · [SR-PACKAGES](../../records/0107_packages.md) decision 3.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
