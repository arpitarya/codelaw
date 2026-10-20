---
type: Handoff
name: W-21
description: "Arpit, 2026-10-20: engine features some ported rules need."
item: W-21
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-21 — Close the engine gaps that hold rules out of the packs

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Lint `` css`…` `` / `` html`…` `` tagged templates as embedded regions | CSS-in-JS and lit templates checked | ⏳ |
| 2 | Give native rules the file's path, raw bytes and neighbours | `md/relative-links`, `html/no-utf8-bom` and Python package checks become possible | ⏳ |
| 3 | Handle modern CSS that tree-sitter-css rejects (range media queries, `layer()`, percentage keyframes) | `css/media-feature-range-notation` can be `auto` | ⏳ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: a region extractor and a grammar fix both risk off-by-one locations, the main injection risk SR-LINTER names.

## Goal

The three gaps are closed and the rules the packs list as waiting on them are ported.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · templates | Tagged templates are regions when `[lint] tagged_templates = true` (SR-LINTER decision 3). | ⏳ |
| 2 · context | Native rules receive path, bytes and a read-only view of sibling files. | ⏳ |
| 3 · CSS | The CSS pack reads modern CSS through the grammar, or a fork of it, not a side parser. | ⏳ |
| 4 · ports | The waiting rules ship with `codelaw test` green. | ⏳ |

## Blockers

None.

## Record affected

[SR-LINTER](../../records/0106_linter.md) decision 3 · [SR-ARCHITECTURE](../../records/0108_architecture.md) decisions 15b, 16.

## Hazards

- **A grammar fork** is a dependency change: L2 and a record.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
