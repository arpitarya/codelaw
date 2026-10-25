---
type: Handoff
name: W-49
description: "Arpit, 2026-10-25: codelaw findings in the terminal and the browser overlay while `npm run dev` runs, through plugins for Vite, Webpack, Rspack and Next.js built on watch mode."
item: W-49
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-49 — Dev-server plugins — Vite, Webpack, Rspack, Next.js

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Decide the package names and L6's fit (npm-only packages under `@codelaw/*`) | the record is the spec | ⏳ |
| 2 | A shared core: spawn `codelaw check --watch --format json --profile local` (W-44), read NDJSON, de-duplicate | one engine, many hosts | ⏳ |
| 3 | `@codelaw/vite-plugin`: errors to Vite's overlay, warnings to the terminal | Vite | ⏳ |
| 4 | `@codelaw/webpack-plugin`: findings into `compilation.errors`/`warnings`; the same plugin runs in **Rspack** (webpack-compatible plugin API) | Webpack and Rspack | ⏳ |
| 5 | `@codelaw/next`: `withCodelaw(nextConfig)`; under webpack it adds the plugin, under Turbopack it runs the watcher beside the dev server and prints to the terminal | Next.js | ⏳ |
| 6 | Builds: `vite build` / `next build` run one `check --profile ci` and fail on errors (option `failOnError`, default on) | CI-grade builds | ⏳ |

**Arpit, 2026-10-25:** *"Watch mode, let's build it. Then CI report formats, let's build it. Exception report, that is code law exceptions, expiring 30 days, group added by … build it. Baseline burn down over commits, build it. Code law migrate from ESLint, flat config, Ruff, style lint, as well as biome … Remove OXLint. Dev server plugin for RSpack, Vite, Webpack, Next.js. Pre-commit hook, post-commit hook, create work items for all these."*

**Model:** Sonnet — four thin adapters over one spawned process, each with an example app test.

## Goal

- Dev: `local` profile (console.log warns, W-39); build: `ci` profile (fails).
- The plugin never lints in-process: it only talks to the codelaw binary, so findings are byte-identical to the CLI's (L5).

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | an example app per host: a `debugger` shows in the overlay/terminal within a second of saving | ⏳ |
| 2 · | `build` fails on a codelaw error | ⏳ |
| 3 · | killing the dev server kills the watcher (no orphan process) | ⏳ |
| 4 · | Rspack runs the webpack plugin unchanged | ⏳ |

## Blockers

waits on **W-44** (watch mode).

## Record affected

SR-PACKAGES (four npm-only packages) · L6 decision on `@codelaw/*` sub-packages — check whether an npm-only package needs the decider's ruling.

## Hazards

- Turbopack has no webpack-plugin API: Next.js gets terminal output only there until it has one.
- Overlay spam: show errors only; warnings stay in the terminal.

## Reference

[Vite plugin API](https://vite.dev/guide/api-plugin) · [Rspack plugin compatibility](https://rspack.rs/guide/compatibility/plugin)
