---
type: Handoff
name: W-44
description: "Arpit, 2026-10-25: a watch mode that re-checks changed files, and the files that depend on them, on every save; the base for the dev-server plugins (W-49)."
item: W-44
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5), W-44 build"
ball: decider
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
| 1 · | saving a file reprints within L8's `fix_one_file`-class budget on the 10k fixture | ⚠ the re-check yes, the reprint no — new `codelaw bench` context rows on a loaded Mac (load 12–28): `watch_rerun` p95 2.9 / 5.7 ms (the engine, save seen → report); `watch_save_to_print` p95 289 / 273 ms (save → JSON line), most of it the poll (≤ 100 ms) and the 100 ms settle the spec asks for, so no build meets < 100 ms save-to-print with a 100 ms debounce. Graph rules on (js/project + py/project) add ~15 ms. Question 2 below |
| 2 · | deleting the last use of an export in file A reports `graph/no-unused-exports` in file B | ✅ `crates/codelaw-cli/tests/watch.rs` `a_save_rechecks_the_files_that_depend_on_it` (also: a save by rename, a file added, a file removed, each run equal to `check --format json`) |
| 3 · | editing `codelaw.toml` changes the next run's verdict | ✅ `watch.rs` `a_config_edit_changes_the_next_verdict` (and a broken config: an `error` line, then recovery) |
| 4 · | `--format json` gives one valid JSON object per run | ✅ every `watch.rs` JSON test parses each line alone and compares it with `check`'s object |
| 5 · | works on the `win32-*` CI legs | ⏳ for CI — `scripts/smoke.sh` now runs `check --watch` on all eight `smoke` legs (a save to `SMOKE.md` must be re-checked); not run here (no Windows target on this Mac). In-process by construction, no Unix-only call outside `cfg(unix)`; Ctrl-C uses the C runtime's `signal`, which `libc` declares for Windows |

## Blockers

None that stop the build. Two questions for the decider:

1. **`notify` is `CC0-1.0`.** L2 decision 1 admits only OSI-approved licences and CC0 is not one, so it was not taken. The watch polls instead: no dependency, the same on every platform, about a fifth of one core idle on the 10k workload, and up to 100 ms more latency than OS events. Options: (a) keep polling; (b) rule CC0-1.0 in for `notify` (an L2 amendment; its macOS/Linux/Windows deps are MIT/ISC/Apache); (c) a later item writes the three OS backends over `libc`/`windows-sys` (no new licence). The `Snapshot` is the one place that would change.
2. **What DoD 1 times.** The engine's re-check is ~3–6 ms; the save-to-print is ~230–290 ms because the spec's own ~100 ms debounce plus the poll come first. Is DoD 1 met by the re-check (`watch_rerun`), or must save-to-print fit < 100 ms (then the debounce must go below 100 ms, or OS events in, or both)?

## What is left

- DoD 5 on CI's `win32-*` `smoke` legs.
- The two questions above.
- `watch_rerun` and `watch_save_to_print` measured on CI's `ubuntu-24.04` runner (they are context rows; nothing gates them).

## Record affected

SR-ARCHITECTURE decision 22 · SR-CLI-MCP decision 2 (the flag) · L8 (a `watch_rerun` measurement).

## Hazards

- Editors save via rename-and-replace: watch the folder, not the inode.
- A `git checkout` touches thousands of files at once: coalesce to one full run.
- `--watch --fix` and format-on-save can loop; detect a write codelaw made itself and skip it.

## Reference

[notify](https://github.com/notify-rs/notify) · `tsc --watch` · `ruff check --watch`
