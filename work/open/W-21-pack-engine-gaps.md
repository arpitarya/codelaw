---
type: Handoff
name: W-21
description: "Arpit, 2026-10-20: engine features some ported rules need."
item: W-21
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-22
updated_by: "Claude Code (Opus 5.5), subagent on branch W-21-pack-gaps"
ball: agent
---

# W-21 — Close the engine gaps that hold rules out of the packs

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Lint `` css`…` `` / `` html`…` `` tagged templates as embedded regions | CSS-in-JS and lit templates checked | ✅ 2026-10-22, on branch `W-21-pack-gaps` |
| 2 | Give native rules the file's path, raw bytes and neighbours | `md/relative-links`, `html/no-utf8-bom` and Python package checks become possible | ✅ 2026-10-22, on branch `W-21-pack-gaps` |
| 3 | Handle modern CSS that tree-sitter-css rejects (range media queries, `layer()`, percentage keyframes) | `css/media-feature-range-notation` can be `auto` | ⏳ not started — needs a grammar fork (see Left) |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: a region extractor and a grammar fix both risk off-by-one locations, the main injection risk SR-LINTER names.

## Goal

The three gaps are closed and the rules the packs list as waiting on them are ported.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · templates | Tagged templates are regions when `[lint] tagged_templates = true` (SR-LINTER decision 3). | ✅ 2026-10-22 |
| 2 · context | Native rules receive path, bytes and a read-only view of sibling files. | ✅ 2026-10-22 |
| 3 · CSS | The CSS pack reads modern CSS through the grammar, or a fork of it, not a side parser. | ⏳ |
| 4 · ports | The waiting rules ship with `codelaw test` green. | 🟡 2026-10-22: `md/relative-links`, `html/no-utf8-bom`, `node/hashbang` ship green; `css/media-feature-range-notation`'s fix waits on row 3 |

## What was built (2026-10-22)

- **Tagged templates** (`crates/codelaw-lang/src/regions.rs`): `` css`…` `` / `` html`…` `` with a plain identifier tag become CSS / HTML regions when `[lint] tagged_templates = true`. Each `${…}` is a *hole*: a placeholder chosen by position keeps the region parseable; a finding inside a placeholder is dropped and an edit that would touch a hole is refused. Nested templates are regions of their own. SR-LINTER decision 3.
- **File context** (`crates/codelaw-engine/src/files.rs`): native rules get the file's path, its bytes (BOM included) and a read-only view of the project (exists, is-dir, read, list), from disk or from the in-memory set for `check_files`/`codelaw test`; paths with `..` answer no. Every question is cached with its answer's hash and a cached result is reused only while all answers hold. `check_source`/`fix_source` read and keep a leading BOM. `codelaw test` checks a file-scope snippet with a file header as a set of files. SR-ARCHITECTURE decision 15b, SR-RULE-AUTHORING decision 4.
- **Rules:** `md/relative-links` (data, `md/recommended`), `html/no-utf8-bom` (html, `html/recommended`), `node/hashbang` (js, `node/recommended`, reads the nearest `package.json`'s `bin`; `suggest`). SR-RULE-PACKS and the pack READMEs updated.
- **Verified:** `cargo clippy --workspace --all-targets -D warnings` clean; `cargo test --workspace` 393 passed, 0 failed; `codelaw test packs/{data,html,python}` 0 failed; `packs/js` and `packs/css` each have one failure that is on `main` already (`js/no-irregular-whitespace`, `css/media-query-no-invalid`: W-23's layout refusal), not this item's. End-to-end: a `css` template's `colr` and `b {}` are reported at host positions and `fix` removes the empty block; a relative link to a missing file stops firing once the file exists, through the cache.

## Left

- **Row 3 (modern CSS) is not started.** DoD 3 rules out a side parser, so it needs a fork of `tree-sitter-css` 0.25.0 (range media queries, `layer()` in `@import`, percentage keyframe selectors), regenerated with the tree-sitter CLI, which is not installed here. A fork is a dependency change (L2 and a record, per Hazards): **the decider** chooses fork-in-tree vs. an upstream patch before an agent starts it.
- Python package checks (e.g. a missing `__init__.py`) are now possible but no rule is filed for them.
- The `IMPLEMENTATION.md` row is written when the branch lands on `main`.

## Blockers

None.

## Record affected

[SR-LINTER](../../records/0106_linter.md) decision 3 · [SR-ARCHITECTURE](../../records/0108_architecture.md) decisions 15b, 16.

## Hazards

- **A grammar fork** is a dependency change: L2 and a record.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
