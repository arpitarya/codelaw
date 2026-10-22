---
type: Handoff
name: W-12
description: "Arpit, 2026-10-20: the structured-file rule form for JSON, YAML and TOML."
item: W-12
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-22
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-12 — Build `target:` + `assert:` rules and the `sort-keys`/`ensure-key`/`when` fix primitives

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Add the `target:` + `assert:` rule form for JSON, YAML and TOML | path assertions on config files | ✅ 2026-10-22 |
| 2 | Add the `sort-keys`, `ensure-key` and `when`/`else` fix primitives | auto fixes for config files | ✅ 2026-10-22 |
| 3 | Port `rules-data`'s waiting rules (SchemaStore schemas, sorting) | the data pack's "not ported yet" list shrinks | ✅ 2026-10-22 |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Sonnet: the form and primitives are specified in SR-RULE-FORMAT, and each ported rule's examples are its test.

## Goal

Rules can assert on paths in JSON/YAML/TOML and fix them with the three primitives; the data pack's schema and sorting rules ship.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · form | `target:` + `assert:` loads and runs (SR-RULE-FORMAT decisions 3 and 3c). | ✅ 2026-10-22 — written down as decision 3e; `Matcher::Target` runs once per file on the data tree; engine tests `target_rules_*` |
| 2 · primitives | `sort-keys`, `ensure-key`, `when`/`else`, each verified like any fix. | ✅ 2026-10-22 — steps run in order; each fix verified (parses, assertion holds, other data unchanged), then the fix loop's own verifier; `structured::tests` (12) |
| 3 · pack | The rules `packs/data/README.md` lists as waiting on this are ported, with `codelaw test` green. | ✅ 2026-10-22 — five `*-schema` rules and `json/package-deps-sorted`; `codelaw test packs/data`: 36 rules, 251 checks, 0 failed |

## Blockers

None.

## Record affected

[SR-RULE-FORMAT](../../records/0112_rule-format.md) decisions 3c and 3e · [SR-LINTER](../../records/0106_linter.md) decision 2 · [SR-ARCHITECTURE](../../records/0108_architecture.md) decision 15 · [SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 7 (the SchemaStore tables, already covered).

## Hazards

- **Comments and key order** in YAML/TOML must survive a fix byte for byte outside the edited range.

## Log

- **2026-10-22 — finished in place (Claude Code).** The WIP merged into `main` built and its tests passed; what was missing:
  - **Steps ran on the original text,** so `[ensure-key, sort-keys]` left the new key unsorted, and `sort-keys` on a sorted or missing target failed the whole fix. Now each step runs on the text the one before left (re-parsed), a step with nothing to do is skipped, and the fix is one edit. `remove` of an array item is verified by the data check, not by the item now at its index.
  - **Markdown front matter and code blocks** are YAML/JSON/TOML regions, so a `target:` rule without `files:` reported on every rule file's front matter. A `target:` rule now runs in a region only with `host:`.
  - **A panic building the data tree** was dropped silently; it is now `codelaw/internal-error` for each `target:` rule.
  - **The W-11 side-by-side merge** (`Matcher::Graph` beside `Matcher::Target` in `compile.rs`, `lint.rs`, `project.rs`) is correct: graph rules never reach `lint_text`, their fixes compile with the query, and a `target:` rule's fix is parsed only as data steps.
  - **Decision 3e did not exist** though the code and the pack README cite it; it is written now.
  - Checked on real files: the repo's workflows, `packaging/vscode/package.json` and `tsconfig.json`, `packaging/python/pyproject.toml` — no schema finding.

## Reference

- [`work/OPEN-WORK.md`](../../work/OPEN-WORK.md): this item's row.
