---
type: Handoff
name: W-14
description: "Arpit, 2026-10-20: the optional external linters SR-LINTER decision 2 allows."
item: W-14
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-22
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-14 — Build the adapters — oxlint, Ruff, stylelint, html-validate

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Build `codelaw-adapters` with one manifest per adapter (tested version range, output mapping) | external findings in the same stream | ✅ 2026-10-22: `crates/codelaw-adapters` (manifests, lookup, versions, runner, four parsers); `codelaw-engine/src/adapters.rs` runs them in `check` and `fix`, cached per file |
| 2 | Map each adapter's output to the three fix classes | L4 holds for adapter findings | ✅ 2026-10-22: Ruff safe/unsafe/none → auto/suggest/manual, stylelint edit-info → auto, oxlint and html-validate manual with docs; `fix` applies them under the verifier |
| 3 | Keep adapters out of hook and LSP mode | SR-ARCHITECTURE decision 19 | ✅ 2026-10-22: only the CLI sets `EngineOptions::adapters`; `--no-adapters`; MCP twins carry it |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Sonnet: each adapter's JSON output is documented and its mapping is checked by fixtures.

## Goal

Installed adapters run from the CLI and CI only, and their findings merge, dedupe and sort with codelaw's own.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · manifests | One manifest per adapter, with its tested version range. | ✅ `tests/manifests.rs` |
| 2 · mapping | Fixtures per adapter: output → diagnostics with a fix class. | ✅ `tests/mapping.rs` (real output of the four pinned tools); `tests/run.rs` replays it through a stand-in program and runs each real tool where installed (all four checked here through `npx`/`uvx` wrappers) |
| 3 · fences | Never run in hook or LSP mode; never bundled; each passes L2 (yamllint stays out). | ✅ in code and `crates/codelaw-cli/tests/adapters.rs`; the full workspace test (385 tests) and clippy run passed on merged `main`, 2026-10-22 |

## Blockers

None. **Verified (2026-10-22):** merged into `main`; the full workspace test (385 tests) and `clippy -D warnings` pass. Closed.

## Record affected

[SR-LINTER](../../records/0106_linter.md) decision 2 · [SR-ARCHITECTURE](../../records/0108_architecture.md) decisions 19 and 28.

## Hazards

- **Duplicates with pack rules:** a finding both report is shown once (same file, range, rule mapping).

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
