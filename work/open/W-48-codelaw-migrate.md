---
type: Handoff
name: W-48
description: "Arpit, 2026-10-25: one command that turns an existing ESLint flat config, Ruff, stylelint or Biome setup into codelaw.toml plus a report of what was mapped, what goes to an adapter, and what was dropped and why."
item: W-48
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-48 — `codelaw migrate` — from ESLint flat config, Ruff, stylelint and Biome

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Define `migrate` in SR-CLI-MCP; settle where it meets SR-AGENT-SURFACES' P2 `codelaw-migrate` skill (the command maps configs; the skill keeps prose → rules) | the record is the spec | ⏳ |
| 2 | **Read the source config:** ESLint flat config and stylelint JS configs through the tool's own `--print-config` (needs Node; refused with a message without it); Ruff `ruff.toml`/`pyproject.toml` and Biome `biome.json(c)` parsed directly | the resolved settings, not a guess | ⏳ |
| 3 | **Map** each rule through `ported_from:` in rule frontmatter; per-glob settings to `[[overrides]]`; ignores to `exclude`; formatter keys through `format --import-config` (Biome's formatter added) | a working `codelaw.toml` | ⏳ |
| 4 | **Report:** mapped · to an adapter · not ported (with the pack README's reason) · options that changed meaning | nothing silently dropped | ⏳ |
| 5 | `--dry-run` prints; `--write` writes; `--check` compares a run of both tools on the repo and lists findings only one side reports | proof the migration is faithful | ⏳ |

**Arpit, 2026-10-25:** *"Watch mode, let's build it. Then CI report formats, let's build it. Exception report, that is code law exceptions, expiring 30 days, group added by … build it. Baseline burn down over commits, build it. Code law migrate from ESLint, flat config, Ruff, style lint, as well as biome … Remove OXLint. Dev server plugin for RSpack, Vite, Webpack, Next.js. Pre-commit hook, post-commit hook, create work items for all these."*

**Model:** Opus — the mapping is judgment per rule, and a wrong mapping passes silently.

## Goal

- `codelaw migrate --from eslint|ruff|stylelint|biome [--write] [--format text|json|markdown]`; several `--from` at once for a mixed repo.
- oxlint is **out of scope** (the decider, 2026-10-25).

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | a fixture per tool: migrate, then `check` reports the same violations as the source tool, except those the report lists | ⏳ |
| 2 · | every source rule appears in the report in exactly one bucket | ⏳ |
| 3 · | `--write` never overwrites an existing `codelaw.toml` without `--force` | ⏳ |
| 4 · | PyPI install without Node: ESLint/stylelint migration exits 2 naming Node; Ruff and Biome work | ⏳ |

## Blockers

None.

## Record affected

SR-CLI-MCP (the command) · SR-AGENT-SURFACES (the P2 skill's split) · SR-RULE-FORMAT decision 2 (`ported_from:` as the mapping table).

## Hazards

- Flat configs are code: never evaluate them in codelaw's process (L5) — only the tool's own `--print-config`, per distinct glob set.
- Plugin rules with no codelaw port go to the adapter bucket, not dropped.

## Reference

[ESLint `--print-config`](https://eslint.org/docs/latest/use/command-line-interface) · [Biome migrate](https://biomejs.dev/guides/migrate-eslint-prettier/)
