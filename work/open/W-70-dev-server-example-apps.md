---
type: Handoff
name: W-70
description: "Arpit, 2026-10-26: a separate item for the example apps of the dev-server plugins — one tiny app each for real Vite, webpack, Rspack and Next.js; W-49's DoD 1 and 4."
item: W-70
filed: 2026-10-26
filed_by: "Arpit (chat), written by Claude Code (Sonnet 5.5)"
updated: 2026-10-26
updated_by: "Claude Code (Sonnet 5.5)"
ball: agent
---

# W-70 — Example apps for the dev-server plugins

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | One tiny app per host under `examples/dev-servers/` (Vite, webpack, Rspack, Next.js), each wired to its `@codelaw/*` plugin | the plugins run on the real hosts | 🔨 built, not yet run on a real host |
| 2 | A CI job (`dev-servers` in `ci.yml`) installs the hosts on the runner via `examples/dev-servers/run.mjs`, saves a file holding a `debugger`, and asserts the finding reaches the terminal (and the dev server's websocket, which feeds the overlay) within a second | W-49 DoD 1 | 🔨 written, first CI run pending |
| 3 | The Rspack app loads `@codelaw/webpack-plugin` unchanged | W-49 DoD 4 | 🔨 `rspack.config.js` imports the same package with the same options as `webpack.config.js`; unrun |

**Arpit, 2026-10-26:** *"create a separate work item for example app"* (the ruling that closed W-49 and filed this).

## Goal

Prove on the real hosts what `plugins.test.js` proves against fakes: a `debugger` shows in the overlay (Vite, webpack, Rspack, Next.js under webpack) or the terminal (Next.js under Turbopack) within a second of saving, and Rspack runs the webpack plugin with no change to it.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | an example app per host (Vite, webpack, Rspack, Next.js): saving a `debugger` shows it in the overlay/terminal within a second | 🔨 apps written (plus a Next.js-on-Turbopack run); not yet proven on a real host |
| 2 · | Rspack runs `@codelaw/webpack-plugin` unchanged (same package, same options) | 🔨 config written; unrun |
| 3 · | the check runs in CI on a runner that installs the hosts, with no host in any repo manifest or lockfile | 🔨 the examples carry no `package.json`; the driver writes one in a temp dir; first CI run pending |

## Blockers

**L11 ([SR-LAW-11](../../records/0013_LAW-11-no-dependencies.md)).** Vite, webpack, Rspack and Next.js are third-party; none is in `DEPENDENCIES.md`. Two routes:

1. **Recommended: a CI job that installs them on the runner.** No repo dependency, nothing to ratify. The item is `agent` and **starts with this route** unless this file records a decider ruling otherwise.
2. **Alternative: the decider ratifies the four as dev-only dependencies of the examples in `DEPENDENCIES.md`**, by name, one by one. Then the examples carry their own manifests and run locally.

No decider ruling is recorded; the item proceeds on route 1 (no blocker, so no `BLOCKED.json`).

## Built 2026-10-26 (what is left)

`examples/dev-servers/{vite,webpack,rspack,next}` (config + a one-file source each, no manifest) and `run.mjs`, the driver the `dev-servers` CI job runs per host (`vite`, `webpack`, `rspack`, `next`, `next-turbopack`) on `ubuntu-latest`, Node 22, after `package`: it installs the host plus the plugin and `@codelaw/rules-js` from the `dist` tarballs into a temp dir, takes the binary out of the linux-x64-gnu CLI tarball, starts the dev server, saves `debugger;`, and times the save to the first finding against a 1000 ms budget. **Nothing has run on a real host yet** (this machine is over the disk budget, L9, so no host was installed); host version ranges (`vite ^7`, `webpack ^5`, `webpack-dev-server ^5`, `@rspack/core ^1`, `next ^15`) are guesses to be corrected by the first CI run. Left: push, read the five jobs, fix what the real hosts disagree on, then close.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) decision 3a (the plugins) · [SR-LAW-11](../../records/0013_LAW-11-no-dependencies.md) (only if route 2 is chosen).

## Hazards

- Whatever the job installs must stay out of every manifest and lockfile the L11 check (`crates/codelaw-cli/tests/dependencies.rs`) reads; install in the job's temp directory, not in the repo.
- Timing ("within a second") on a shared runner is noisy: measure from the save to the first finding, and state the runner.
- Turbopack has no webpack-plugin API: Next.js shows terminal output only there.

## Reference

[W-49](../../archive/open/W-49-dev-server-plugins.md) (closed; built the plugins) · [Vite plugin API](https://vite.dev/guide/api-plugin) · [Rspack plugin compatibility](https://rspack.rs/guide/compatibility/plugin)
