---
type: Handoff
name: W-12
description: "Arpit, 2026-10-20: the structured-file rule form for JSON, YAML and TOML."
item: W-12
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-12 — Build `target:` + `assert:` rules and the `sort-keys`/`ensure-key`/`when` fix primitives

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Add the `target:` + `assert:` rule form for JSON, YAML and TOML | path assertions on config files | ⏳ |
| 2 | Add the `sort-keys`, `ensure-key` and `when`/`else` fix primitives | auto fixes for config files | ⏳ |
| 3 | Port `rules-data`'s waiting rules (SchemaStore schemas, sorting) | the data pack's "not ported yet" list shrinks | ⏳ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Sonnet: the form and primitives are specified in SR-RULE-FORMAT, and each ported rule's examples are its test.

## Goal

Rules can assert on paths in JSON/YAML/TOML and fix them with the three primitives; the data pack's schema and sorting rules ship.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · form | `target:` + `assert:` loads and runs (SR-RULE-FORMAT decisions 3 and 3c). | ⏳ |
| 2 · primitives | `sort-keys`, `ensure-key`, `when`/`else`, each verified like any fix. | ⏳ |
| 3 · pack | The rules `packs/data/README.md` lists as waiting on this are ported, with `codelaw test` green. | ⏳ |

## Blockers

None.

## Record affected

[SR-RULE-FORMAT](../../records/0112_rule-format.md) decisions 3 and 3c · [SR-RULE-PACKS](../../records/0101_rule-packs.md).

## Hazards

- **Comments and key order** in YAML/TOML must survive a fix byte for byte outside the edited range.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
