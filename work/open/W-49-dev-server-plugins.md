---
type: Handoff
name: W-49
description: "Arpit, 2026-10-25: codelaw findings in the terminal and the browser overlay while `npm run dev` runs, through plugins for Vite, Webpack, Rspack and Next.js built on watch mode."
item: W-49
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Claude Code (Sonnet 5.5)"
ball: agent
---

# W-49 — Dev-server plugins — Vite, Webpack, Rspack, Next.js

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Decide the package names and L6's fit (npm-only packages under `@codelaw/*`) | the record is the spec | ✅ 2026-10-26 — three packages, not four: Rspack runs the webpack plugin. SR-PACKAGES decision 3a; they fit L6 as `@codelaw/*` (no ruling needed beyond Arpit's 2026-10-25 request) |
| 2 | A shared core: spawn `codelaw check --watch --format json --profile local` (W-44), read NDJSON, de-duplicate | one engine, many hosts | ✅ `packaging/npm/dev-core/core.js`, copied into each package by `package.py sync-dev-core`, because a package may have no dependency |
| 3 | `@codelaw/vite-plugin`: errors to Vite's overlay, warnings to the terminal | Vite | ✅ `packaging/npm/vite-plugin` |
| 4 | `@codelaw/webpack-plugin`: findings into `compilation.errors`/`warnings`; the same plugin runs in **Rspack** (webpack-compatible plugin API) | Webpack and Rspack | ✅ `packaging/npm/webpack-plugin` (a fake compiler with the hooks Rspack shares; no real Rspack run) |
| 5 | `@codelaw/next`: `withCodelaw(nextConfig)`; under webpack it adds the plugin, under Turbopack it runs the watcher beside the dev server and prints to the terminal | Next.js | ✅ `packaging/npm/next` |
| 6 | Builds: `vite build` / `next build` run one `check --profile ci` and fail on errors (option `failOnError`, default on) | CI-grade builds | ✅ `failOnError` (default on) in all three |

**Arpit, 2026-10-25:** *"Watch mode, let's build it. Then CI report formats, let's build it. Exception report, that is code law exceptions, expiring 30 days, group added by … build it. Baseline burn down over commits, build it. Code law migrate from ESLint, flat config, Ruff, style lint, as well as biome … Remove OXLint. Dev server plugin for RSpack, Vite, Webpack, Next.js. Pre-commit hook, post-commit hook, create work items for all these."*

**Model:** Sonnet — four thin adapters over one spawned process, each with an example app test.

## Goal

- Dev: `local` profile (console.log warns, W-39); build: `ci` profile (fails).
- The plugin never lints in-process: it only talks to the codelaw binary, so findings are byte-identical to the CLI's (L5).

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | an example app per host: a `debugger` shows in the overlay/terminal within a second of saving | ⏳ the debugger-in-overlay path is tested per host against a fake of its API (`plugins.test.js`); an example app on real Vite, webpack, Rspack and Next.js needs those installed — L11 forbids them as dependencies, so it is a decider-approved dev install or a CI job |
| 2 · | `build` fails on a codelaw error | ✅ `plugins.test.js`: Vite `buildStart`, the webpack `run` hook and Next's Turbopack build all fail on `js/no-debugger` |
| 3 · | killing the dev server kills the watcher (no orphan process) | ✅ `plugins.test.js`: `stop()` / server close / `watchClose` kill the child (`process.kill(pid, 0)` fails); the child is also killed on the host's `exit` |
| 4 · | Rspack runs the webpack plugin unchanged | ⏳ the plugin uses only hooks and `compilation.errors`/`warnings` that Rspack documents; not run on a real Rspack |

## Blockers

None. W-44 (watch mode) has landed.

## What is left

- DoD 1 and 4 on the real hosts: an example app each for Vite, webpack, Rspack and Next.js (webpack, Rspack, Vite and Next.js are third-party packages; L11 needs the decider to ratify them as dev dependencies of the examples, or the check runs in a CI job that installs them).
- The first publish of the three packages and `scripts/npm-trust.sh` for them (the decider; no agent publishes).

## Record affected

SR-PACKAGES (four npm-only packages) · L6 decision on `@codelaw/*` sub-packages — check whether an npm-only package needs the decider's ruling.

## Hazards

- Turbopack has no webpack-plugin API: Next.js gets terminal output only there until it has one.
- Overlay spam: show errors only; warnings stay in the terminal.

## Reference

[Vite plugin API](https://vite.dev/guide/api-plugin) · [Rspack plugin compatibility](https://rspack.rs/guide/compatibility/plugin)
