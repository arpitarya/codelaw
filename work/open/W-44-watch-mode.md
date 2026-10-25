---
type: Handoff
name: W-44
description: "Arpit, 2026-10-25: a watch mode that re-checks changed files, and the files that depend on them, on every save; the base for the dev-server plugins (W-49)."
item: W-44
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-44 — `codelaw check --watch` — re-check on every save

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend SR-ARCHITECTURE decision 22 (the daemon's 'no watcher dependency') and add `--watch` to SR-CLI-MCP | the record is the spec | ⏳ |
| 2 | Check the `notify` crate's licence against L2 and take it in | OS file events on macOS, Linux, Windows | ⏳ |
| 3 | Build the loop: debounce ~100 ms, honour `.gitignore`/`exclude`, re-run file rules on changed files and graph rules on their dependents, reusing the cache | a fresh verdict per save | ⏳ |
| 4 | Output: `text` clears and reprints with a summary line; `json` prints one object per run (NDJSON) for tools | people and plugins both read it | ⏳ |
| 5 | Config, rules and baseline changes reload, as the daemon does | no restart after editing `codelaw.toml` | ⏳ |
| 6 | Docs: USAGE, README command table, CHANGELOG | users find it | ⏳ |

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
| 1 · | saving a file reprints within L8's `fix_one_file`-class budget on the 10k fixture | ⏳ |
| 2 · | deleting the last use of an export in file A reports `graph/no-unused-exports` in file B | ⏳ |
| 3 · | editing `codelaw.toml` changes the next run's verdict | ⏳ |
| 4 · | `--format json` gives one valid JSON object per run | ⏳ |
| 5 · | works on the `win32-*` CI legs | ⏳ |

## Blockers

None.

## Record affected

SR-ARCHITECTURE decision 22 · SR-CLI-MCP decision 2 (the flag) · L8 (a `watch_rerun` measurement).

## Hazards

- Editors save via rename-and-replace: watch the folder, not the inode.
- A `git checkout` touches thousands of files at once: coalesce to one full run.
- `--watch --fix` and format-on-save can loop; detect a write codelaw made itself and skip it.

## Reference

[notify](https://github.com/notify-rs/notify) · `tsc --watch` · `ruff check --watch`
