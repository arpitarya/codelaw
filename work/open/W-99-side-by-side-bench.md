---
type: Handoff
name: W-99
description: "Arpit, 2026-10-27 (W-36 gap G5): measure codelaw against ESLint, Prettier, Biome, oxlint and Ruff on the same pinned repos and runner, before any speed claim."
item: W-99
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5), W-99 step 1 proposed, harness built"
ball: arpit
---

# W-99 — A side-by-side benchmark before any speed claim

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Name each rival at an exact version as a CI-only bench tool, one row each in L11's list (§3 of `records/0013_LAW-11-dependencies.md`) (L11 — the decider ratifies each row by name when it lands) | the tools may be installed in CI | ✅ proposed below (D-37…D-41) · 🔴 Arpit ratifies the rows |
| 2 | A `bench-rivals` CI job (manual and weekly): the three pinned corpus repos, the same runner image, same day; equivalent rule sets (recommended presets), lint and format separately, cold and warm | numbers a reader can reproduce | ✅ built, skipped until step 1 (`scripts/bench-rivals.py`, `perf/rivals.toml`, `.github/workflows/bench-rivals.yml`); dry run with codelaw alone ✅ · ⏳ the first real run |
| 3 | Results in L8 §Measurements as context rows; the README states a claim only from these, with the date and machine | an honest claim | ⏳ |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Sonnet.

Why: every rival publishes a speed claim (Biome "~35x Prettier", oxlint "50–100x ESLint", Ruff "10–100x Flake8"); codelaw has none, and makes none until measured ([proposal](../proposals/linter-for-ai-positioning.md) §4).

## Step 1 — the proposed L11 rows (2026-10-27, not in §3 until ratified)

L11's list has no "proposed" status (`pending` is for a dependency already in use, decision 5), so the rows wait here and in the inbox. Each is a **CI-only bench tool**: installed only by `scripts/bench-rivals.py install` inside `bench-rivals.yml`, in a throwaway folder; never in the engine, a package, a test or `ci.yml`. Versions are each tool's latest on its registry on 2026-10-27; licences and trees read from the registries' metadata (`npm view`, `npm install --package-lock-only --ignore-scripts` in an empty folder — nothing installed or run; PyPI's JSON API).

| id | unit | what it does | pin | licence (L2) | installed tree | status |
|---|---|---|---|---|---|---|
| D-37 | **ESLint** (`eslint` + `@eslint/js`, its recommended rules, published by the ESLint project) | the JS linter timed beside `codelaw check` | `eslint@10.13.0`, `@eslint/js@10.0.1` | MIT; tree: 54 MIT, 12 Apache-2.0, 6 BSD-2-Clause, 4 ISC, 1 BSD-3-Clause, **1 BlueOak-1.0.0** (`minimatch@10.2.6`: OSI-approved and permissive, but not one of L2 decision 2's named examples — say so if you rule it out) | 78 packages | proposed |
| D-38 | **Prettier** | the formatter timed beside `codelaw format --check` | `prettier@3.9.10` | MIT | 1 (no dependencies) | proposed |
| D-39 | **Biome** (`@biomejs/biome` + its platform binary) | lint and format, timed beside both | `@biomejs/biome@2.6.3` | MIT OR Apache-2.0 (binaries too) | 1 + 1 of 10 optional platform binaries | proposed |
| D-40 | **oxlint** (+ its platform binary) | the JS/TS linter timed beside `codelaw check` | `oxlint@1.89.0` | MIT (binaries too) | 1 + 1 of 19 optional platform binaries | proposed |
| D-41 | **Ruff** (the wheel) | Python lint and format, timed beside both | `ruff==0.17.3` | MIT (`license_expression`) | 1 wheel, no dependencies | proposed |

On ratification, these rows go into §3 under a new heading **"CI — the side-by-side bench (W-99)"** with the decider's words as their status; `bench-rivals.py` then lets the tool run (it requires `ratified` and the table's version on the row). `oxfmt` (oxc's formatter) and `typescript-eslint` are **not** proposed: the item names five tools, and without `typescript-eslint` ESLint lints JavaScript only (the table says so and codelaw's comparison row is scoped to match).

## What is built (2026-10-27)

- **`perf/rivals.toml`** — the table: each tool's exact version, licence, L11 row, install spec, recommended rule set, languages, config, and the command for `lint` and `format`. codelaw's own row runs the reference workload's config with `[format] check = false`, so lint and format are timed apart.
- **`scripts/bench-rivals.py`** (standard library, Python 3.11) — `--self-test`, `plan`, `gate`, `install`, `run`. `codelaw bench`'s method: `--runs` (10), first discarded, median and nearest-rank p95, spawn to exit; cold = a new empty cache per run, warm = a primed cache, own process (a tool with no cache — Biome, oxlint — is reported cold only). Each tool gets its own copy of each repo, the repo's own configs for all six tools stripped, its config at the root. codelaw is also timed with `--lang` scoped to each rival's languages — the row to compare a rival with. Writes `results.json` and `results.md` (one table per repo: tool, op, cache, scope, median, p95, min–max, exit codes; plus each tool's version and rule set). **The L11 switch:** `install` and `run` refuse a rival whose row is absent, not `ratified`, or at another version (checked against a fake list: ratified → runs; other version, `pending`, absent → refused).
- **`.github/workflows/bench-rivals.yml`** — `workflow_dispatch` and weekly (Mon 02:30 IST), `ubuntu-24.04`. The gate step runs `--self-test`, `plan` and `gate`; every other step is skipped until `gate` says a rival is ratified. No step names a package; `install` installs exactly the ratified pins.
- **Dry run, codelaw alone** (`bench-rivals.py run --runs 3`, release build of `main` `140492c`, a loaded developer Mac — Apple M4, 10 cores, load 11–14 with other agents compiling): 54 rows over the three repos (674, 246 and 77 files) in 92 s. **These are not numbers to cite** — a loaded machine and 3 runs. One thing seen: `format --check` warm is no faster than cold (e.g. djangoproject.com 2.4 s against 2.0 s median) — it suggests `format --check` gains nothing from a primed cache outside the daemon; not checked further here, and the runner's numbers will say whether it holds.

## Left

1. 🔴 **Arpit:** ratify D-37…D-41 by name (the inbox row), or strike any.
2. Paste the ratified rows into L11 §3 with the decider's words (an agent, in the ruling's change), and decide with the ruling whether the bench tools get ledger lines (`dependencies.rs` reads workflows' install lines; these installs are in the script, so the script's own L11 check is what guards them today).
3. `gh workflow run bench-rivals.yml` — the first real run; fix whatever a rival's command needs on the runner (ESLint's config resolution, Prettier's exit 2 on unparsable files are kept, not hidden).
4. Step 3: the runner's results into L8 §Measurements as context rows; the README states a claim only from them, with date and machine.

## Definition of done

- One CI run produces a table per repo: tool, version, rule set, cold and warm p50/p95; the README cites it or says nothing.

## Record affected

SR-LAW-11 §3 (CI-only rows) · SR-LAW-8 §Measurements (context rows) · SR-CLI-MCP decision 10 (the harness, as built).

## Hazards

- Equal work is hard: compare like rule sets, say where they differ.
- A loaded machine produces a clean anomaly (LESSONS) — runner only.
