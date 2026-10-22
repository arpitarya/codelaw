---
type: Handoff
name: W-13
description: "Arpit, 2026-10-20: SR-BASELINE's grandfathering and the time and change-set inputs L5 names."
item: W-13
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-22
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-13 — Build baselines, the deprecations registry, `--as-of` and the git inputs

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read `HEAD`'s commit date as the as-of default; add `--as-of` | L5: time is an explicit input | ✅ 2026-10-22 |
| 2 | Add `--changed-since` and `--staged` through the `git` binary | SR-ARCHITECTURE decision 14 | ✅ 2026-10-22 |
| 3 | Build count-based baselines with `until`, `on_touch` and the ratchet | grandfather old violations | ✅ 2026-10-22 |
| 4 | Add the deprecations registry and `graph/no-deprecated-usage` | the engine's own rule | ✅ 2026-10-22 — `rewrite` (the `suggest` fix) not applied yet |
| 5 | Check whether count-based baselines let a new use offset a removed one | the veto check (was B-014) | ✅ 2026-10-22 — yes without a change set; closed by `on_touch` when the file is touched |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: deadlines, touch policy and ratchet interact, and the 'offset' hole is a judgment the tests can only sample.

## Goal

Baselines and deprecations work as SR-BASELINE specifies, with time and the change set taken only from explicit inputs.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · time | The as-of date is `HEAD`'s commit date or `--as-of`; never the wall clock (L5). | ✅ 2026-10-22 — `--as-of`, then `CODELAW_AS_OF`, then `git log -1 --format=%cI`; `the_same_repo_on_two_dates_gives_the_same_output` runs one commit with the clock in UTC+14 and UTC−12, a deadline between them, byte-identical |
| 2 · git | `--changed-since <ref>`, `--staged`; without `git` they say so and the rest runs. | ✅ 2026-10-22 — a warning, nothing touched, exit code unchanged; MCP `check_paths`/`fix_paths` take `as_of`, `changed_since`, `staged` (parity test) |
| 3 · baseline | `.codelaw/baseline/<rule>.toml`, count-based, `until`, `on_touch`, ratchet, CODEOWNERS. | ✅ 2026-10-22 — new crate `codelaw-baseline`; `baseline add/prune/check`, `fix` prunes, `codelaw/stale-baseline`; `baseline check --since` exits 1 on any loosening and warns when no CODEOWNERS covers the folder |
| 4 · deprecations | `.codelaw/deprecations.toml` and `graph/no-deprecated-usage` (uses W-11's graph). | ✅ 2026-10-22 — registry, `[deprecation]` defaults, JSDoc/PEP 702 markers, `codelaw deprecate`; aliases and barrels by identity, outside packages by specifier |
| 5 · veto | The offset check is run and its result recorded in SR-BASELINE. | ✅ 2026-10-22 — SR-BASELINE §Veto condition: the swap passes plain `check`, fails once the file is in the change set; open only under `on_touch = keep`/`warn` or a CI without `--changed-since` |

**Left (2026-10-22), none of it in the DoD above:**
- `on_touch_scope = "function"` (decision 3): only `"file"` exists; `[baseline]` is still an unvalidated table.
- The `rewrite = { from, to }` `suggest` fix (decision 6): parsed and checked, never applied; every finding is `manual`.
- `rules_for_path` does not list the deprecations of a file yet (decision 8).
- The LSP applies no baseline: `check_document` shows raw findings.
- `check_snippet` applies the baseline to file rules only, as it runs no project rules.

## Blockers

None. W-11's graph landed on `main` 2026-10-22.

## Record affected

[SR-BASELINE](../../records/0104_baseline-and-deprecations.md) (now owns `crates/codelaw-baseline`) · [SR-LAW-5](../../records/0007_LAW-5-no-model-in-the-engine.md) · [SR-ARCHITECTURE](../../records/0108_architecture.md) decision 14 · [SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decisions 1, 4, 5 · [SR-CONFIG](../../records/0110_config.md) (`[deprecation]`) · [SR-PACKAGES](../../records/0107_packages.md) · [SR-RULE-FORMAT](../../records/0112_rule-format.md) · [SR-RULE-PACKS](../../records/0101_rule-packs.md) · [SR-AGENT-SURFACES](../../records/0102_agent-surfaces.md) (the hook's touched files).

## Hazards

- **The wall clock** must never leak in: a test runs the same repo on two dates and expects the same output.
- **The shared `CARGO_TARGET_DIR`** across worktrees: Cargo hashes workspace crates by their workspace-relative path, so worktrees with the same layout overwrite each other's `codelaw-*` rlibs and a build can link another branch's crate. W-13 was built and tested in its own target dir for that reason.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- `crates/codelaw-cli/tests/baseline.rs`, `crates/codelaw-baseline/src/tests.rs`: the tests.
