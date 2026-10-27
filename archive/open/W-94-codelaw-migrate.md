---
type: Handoff
name: W-94
description: "Arpit, 2026-10-27 (W-36 gap G1): a deterministic codelaw migrate that translates ESLint, Prettier, Biome, Ruff and stylelint configs into codelaw.toml."
item: W-94
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5), W-94 closed"
ball: closed
---

# W-94 — `codelaw migrate` — switch from ESLint, Prettier, Biome, Ruff or stylelint in one command

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `codelaw migrate eslint\|prettier\|biome\|ruff\|stylelint [--write]`: read the tool's config (JSON, YAML, TOML, `.eslintrc*`, `package.json` keys; flat `eslint.config.*` only where it is static), map each rule via `ported_from:` to its codelaw id and options, presets to presets | a `codelaw.toml` a team can review | ✅ `crates/codelaw-cli/src/migrate.rs`; flat configs read through the JS/TS grammar where literal, never run; presets via `data/migrate-presets.json`, Biome via `codelaw-engine/data/biome-sources.json` |
| 2 | A report: mapped, mapped with a changed option, no codelaw equivalent (by name), ignored settings — exit 0, `--format json` | nothing is dropped silently | ✅ `mapped` / `changed` / `unmapped` / `ignored` lines, counts, packs not searched; `--format json`; exit 0, or 2 for no config, a program, a write failure |
| 3 | Inline suppressions: accept `eslint-disable*`, `# noqa`, `biome-ignore` as aliases of `codelaw-ignore` (a setting, on in migrated configs), or rewrite them with `migrate --rewrite-ignores` | the codebase does not need a mass edit on day one | ✅ `[lint] ignore_aliases` (SR-LINTER decision 6), on in every migrated ESLint, Biome and Ruff config; `--rewrite-ignores` not built — the step offers aliases *or* a rewrite, and the aliases make the rewrite unneeded |
| 4 | Docs: a "Coming from …" section per tool in USAGE; the `codelaw-migrate` skill calls the command | the switch is documented | ✅ USAGE *Coming from another tool* (one section per tool); the `codelaw-migrate` skill runs the command first |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../../work/proposals/linter-for-ai-positioning.md).

**Model:** Sonnet.

Why: Biome (`biome migrate eslint/prettier/stylelint`) and oxc (`@oxlint/migrate`, `oxfmt --migrate prettier/biome`) ship one; codelaw has only an agent skill ([proposal](../../work/proposals/linter-for-ai-positioning.md) §3).

## Definition of done

**Closed 2026-10-27 (Claude Code, Opus 5.5).** Evidence: `cargo test -p codelaw-cli --test migrate` (5 passed) runs `migrate --write` on `fixtures/migrate/{eslint-prettier,biome,ruff}` with the packs installed, then `check`: each gives exactly the findings in its `expected.txt`, the report names every rule in its `unmapped.txt`, and a second copy writes byte-identical `codelaw.toml`; a computed `eslint.config.js` and an `.eslintrc.js` exit 2 unread. `expected.txt` is what each tool documents for those rules on those lines — ESLint, Biome and Ruff are not installed on this machine, so the tools themselves were not run; W-99's bench is where they run side by side. SR-CLI-MCP decision 15, SR-LINTER decision 6, SR-CONFIG decision 8, SR-ARCHITECTURE's lint pipeline (step 6).

- On three fixture repos (an ESLint flat + Prettier repo, a Biome repo, a Ruff `pyproject.toml` repo) `migrate --write` produces a config under which `codelaw check` reports the same rule set's findings as the source tool on its ported rules, and the report names every unmapped rule.
- The same input gives byte-identical output (L5); no JavaScript is executed to read a config.

## Record affected

SR-CLI-MCP (the command) · SR-CONFIG (ignore aliases) · SR-BASELINE (inline suppressions).

## Hazards

- A dynamic `eslint.config.js` cannot be read without running it — refuse with a clear message naming what to do; never execute it (L5).
- Option names differ between a source rule and its port — the mapping table is data next to each rule (`ported_from:`), not code.
