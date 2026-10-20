---
type: Handoff
name: W-07
description: "Arpit, 2026-10-20: the formatter L7 requires: codelaw lints but does not format yet, so it may not be released."
item: W-07
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-07 — Build the formatter — `codelaw format` and the format step of `fix`

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Embed the formatter engines SR-FORMATTER names (Biome, Ruff's formatter, pretty_yaml, taplo, dprint-plugin-markdown, markup_fmt) in a `codelaw-format` crate | every L3 language formats | ⏳ |
| 2 | Add `codelaw format` (`--check`, `--diff`), and the format step at the end of `fix` | the command L7 and L6 decision 3 name | ⏳ |
| 3 | Make `check` report unformatted files | L7 decision 2's one binary, one config | ⏳ |
| 4 | Assert, over the L3 fixture matrix, that every language has a lint rule and a formatter that round-trips its fixtures | L7 checked by a test (was B-017) | ⏳ |
| 5 | Diff codelaw's output against Prettier and Black on a pinned corpus | how close the defaults are (was B-015) | ⏳ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: embedding six engines behind one verifier, where a formatter that changes meaning is the failure and no existing test catches it.

## Goal

`codelaw format` formats every L3 language with SR-FORMATTER's engines and defaults; `fix` ends with a format; `check` reports unformatted files.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · engines | Each engine SR-FORMATTER decision 2 names is compiled in; each passes [L2](../../records/0004_LAW-2-foss-dependencies.md) (`crates/codelaw-cli/tests/licences.rs`). | ⏳ |
| 2 · command | `codelaw format [--check] [--diff] <paths>`; exit codes as `check`. | ⏳ |
| 3 · verify | A format that changes meaning or breaks the parse is rolled back and reported (SR-FORMATTER decision 4). | ⏳ |
| 4 · L7 test | The fixture-matrix assertion of [SR-LAW-7](../../records/0009_LAW-7-linter-and-formatter.md) §How L7 is enforced runs in `cargo test`. | ⏳ |
| 5 · gate | `python3 scripts/law-gate.py --binary target/release/codelaw` no longer lists `format`. | ⏳ |
| 6 · parity | A corpus diff against Prettier and Black is recorded in SR-FORMATTER's Consequences. | ⏳ |

## Blockers

None.

## Record affected

[SR-FORMATTER](../../records/0105_formatter.md) · [SR-LAW-7](../../records/0009_LAW-7-linter-and-formatter.md) §How L7 is enforced · [SR-PACKAGES](../../records/0107_packages.md) decision 1 (`codelaw-format`).

## Hazards

- **Biome is git-pinned, not on crates.io** (SR-FORMATTER): pin a commit, record it, and keep it in the cache key.
- **[L8](../../records/0010_LAW-8-performance-budgets.md):** formatting runs on every `fix`; it is in `fix_one_file`.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- [`scripts/law-gate.py`](../../scripts/law-gate.py): the release gate this item clears for `format`.
