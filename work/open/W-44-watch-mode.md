---
type: Handoff
name: W-44
description: "Arpit, 2026-10-25: a watch mode that re-checks changed files, and the files that depend on them, on every save; the base for the dev-server plugins (W-49)."
item: W-44
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-44 — `codelaw check --watch` — re-check on every save

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend SR-ARCHITECTURE decision 22 (the daemon's 'no watcher dependency') and add `--watch` to SR-CLI-MCP | the record is the spec | ✅ 2026-10-25 — SR-ARCHITECTURE 22 and new 22a; SR-CLI-MCP 1, 2 and 10; SR-CONFIG 10 (`--watch` defaults to `local`); SR-LINTER 2 (where the ported dedupe runs); SR-PACKAGES 11 (the smoke's watch step); L8 §Measurements (context rows) |
| 2 | Check the `notify` crate's licence against L2 and take it in | OS file events on macOS, Linux, Windows | ❌ refused by L2 — every `notify` release (4.x to 9.0.0-rc.6) is `CC0-1.0`, not OSI-approved (L2 decision 1; SR-RULE-PACKS already refuses CC0 data). Built instead with **no dependency**: the engine polls file metadata every 100 ms (`codelaw-engine/src/watch.rs`, SR-ARCHITECTURE 22a). Question 1 below |
| 3 | Build the loop: debounce ~100 ms, honour `.gitignore`/`exclude`, re-run file rules on changed files and graph rules on their dependents, reusing the cache | a fresh verdict per save | ✅ 2026-10-25 — `Watch` keeps `check`'s parts (file results, graph sources and facts, adapter findings) and `check` and the watch share `assemble`; a save re-checks the file, any file whose cached answers it changed, its graph facts, then every graph rule; added/removed files re-walk; > 256 changes (a checkout) is one full run; settle 100 ms, at most 2 s |
| 4 | Output: `text` clears and reprints with a summary line; `json` prints one object per run (NDJSON) for tools | people and plugins both read it | ✅ 2026-10-25 — text clears on a TTY and ends `watching N files · run K · <file> changed · Ctrl-C to stop`; json is `check`'s object + `watch {run, watched, changed[], reloaded}` (+ `fixed[]`), an unloadable config is an `error` line |
| 5 | Config, rules and baseline changes reload, as the daemon does | no restart after editing `codelaw.toml` | ✅ 2026-10-25 — also `.editorconfig`, ignore files, `.codelaw/`, `package.json`/`pyproject.toml`; a config that fails to load is reported and retried on the next change |
| 6 | Docs: USAGE, README command table, CHANGELOG | users find it | ✅ 2026-10-25 — USAGE's command table and a "While you work" bullet; README's roadmap row (it has no command table); CHANGELOG *Unreleased*; `docs-sync.py --check` green |

**Arpit, 2026-10-25:** *"Watch mode, let's build it. Then CI report formats, let's build it. Exception report, that is code law exceptions, expiring 30 days, group added by … build it. Baseline burn down over commits, build it. Code law migrate from ESLint, flat config, Ruff, style lint, as well as biome … Remove OXLint. Dev server plugin for RSpack, Vite, Webpack, Next.js. Pre-commit hook, post-commit hook, create work items for all these."*

**Model:** Opus — it reverses a record decision and must hold L8's per-file budget across a graph; close call, Sonnet could build it from this file.

## Goal

- `codelaw check --watch [paths]` and `codelaw fix --watch` (the second warns: it can fight an editor's own save).
- Profile `local` by default (W-39); `--profile` overrides.
- In-process, so it works on Windows too (the daemon does not).
- Ctrl-C exits 0.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | the re-check (`watch_rerun`) within L8's `fix_one_file`-class budget on the 10k fixture — save-to-print is context only (the decider, 2026-10-26) | ✅ re-check;  the re-check yes, the reprint no — new `codelaw bench` context rows on a loaded Mac (load 12–28): `watch_rerun` p95 2.9 / 5.7 ms (the engine, save seen → report); `watch_save_to_print` p95 289 / 273 ms (save → JSON line), most of it the poll (≤ 100 ms) and the 100 ms settle the spec asks for, so no build meets < 100 ms save-to-print with a 100 ms debounce. Graph rules on (js/project + py/project) add ~15 ms. Question 2 below |
| 2 · | deleting the last use of an export in file A reports `graph/no-unused-exports` in file B | ✅ `crates/codelaw-cli/tests/watch.rs` `a_save_rechecks_the_files_that_depend_on_it` (also: a save by rename, a file added, a file removed, each run equal to `check --format json`) |
| 3 · | editing `codelaw.toml` changes the next run's verdict | ✅ `watch.rs` `a_config_edit_changes_the_next_verdict` (and a broken config: an `error` line, then recovery) |
| 4 · | `--format json` gives one valid JSON object per run | ✅ every `watch.rs` JSON test parses each line alone and compares it with `check`'s object |
| 5 · | works on the `win32-*` CI legs | ⏳ for CI — `scripts/smoke.sh` now runs `check --watch` on all eight `smoke` legs (a save to `SMOKE.md` must be re-checked); not run here (no Windows target on this Mac). In-process by construction, no Unix-only call outside `cfg(unix)`; Ctrl-C uses the C runtime's `signal`, which `libc` declares for Windows. 2026-10-27: CI's `build win32-*` legs had been red since 2026-10-26 (a grammar scanner's `iswspace`, not the watch), so `smoke` never ran; fixed in the grammars' `build.rs` (IMPLEMENTATION 2026-10-27, W-44 CI fix) |


## Rulings

**Arpit, 2026-10-26 (docket):** watch events — *"Keep polling"*; what must be under 100 ms — *"The re-check"*. So: no `notify`, L2 unchanged; DoD 1 is judged on `watch_rerun`, and save-to-print stays a context measurement. OS backends over `libc`/`windows-sys` would be a later item if latency ever matters.

## Blockers

None. Both questions the build raised were ruled by the decider on 2026-10-26 (see Rulings): keep polling (no `notify`, L2 unchanged) and DoD 1 is judged on the re-check. An OS-event backend over `libc`/`windows-sys` would be a later item only if latency ever matters.

## What is left

- DoD 5 on CI's `win32-*` `smoke` legs (`scripts/smoke.sh` runs `check --watch`; no Windows target on this Mac). The watch step passed on both `win32` legs in run 40388001095 (IMPLEMENTATION 2026-10-27); the `smoke` job stays red for another reason (npm and PyPI output not byte-equal, all eight legs) — W-84. Close on the first green `smoke` run.
- `watch_rerun` and `watch_save_to_print` measured on CI's `ubuntu-24.04` runner (context rows; nothing gates them).
- Re-verified locally 2026-10-26: `cargo test -p codelaw-cli --test watch` 6 passed, 0 failed.

## Record affected

SR-ARCHITECTURE decision 22 · SR-CLI-MCP decision 2 (the flag) · L8 (a `watch_rerun` measurement).

## Hazards

- Editors save via rename-and-replace: watch the folder, not the inode.
- A `git checkout` touches thousands of files at once: coalesce to one full run.
- `--watch --fix` and format-on-save can loop; detect a write codelaw made itself and skip it.

## Reference

[notify](https://github.com/notify-rs/notify) · `tsc --watch` · `ruff check --watch`
