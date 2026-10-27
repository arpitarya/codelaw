---
type: Proposal
title: Type-aware rules through the project's own TypeScript 7.1
description: "W-101's outcome: type-aware TS rules (no-floating-promises, no-unsafe-*, switch-exhaustiveness …) and TypeScript's own errors, answered by a small Node helper that drives the project's installed TypeScript 7.1 API; preview on the 7.1 RC, stable after 7.1 GA; the oxlint adapter covers type rules until then. Ruled by the decider 2026-10-27."
status: graduated
timestamp: 2026-10-27T12:40:00+05:30
---

# Type-aware rules through the project's own TypeScript 7.1

**Ruled 2026-10-27** (the decider, by voice, Cowork) → built by [W-104](../open/W-104-type-aware-records.md) (records: the design of record is [SR-TYPES](../../records/0117_types.md)), [W-105](../open/W-105-type-aware-engine.md) (helper, engine, rules), [W-106](../open/W-106-typescript-version-watch.md) (version watch), [W-107](../open/W-107-oxlint-type-aware-gap.md) (the gap filler). First parked 2026-10-25 as B-049 (§6); reopened as research W-101 on W-36's R3.

## 1 · In one line

**codelaw ships a small Node helper that loads the project's own TypeScript (≥ 7.1) once and answers a fixed set of type questions; rules in the existing `ts` pack ask them through a `type:` clause. Preview on the 7.1 RC (2026-11-10), stable once 7.1 is GA (2026-11-24). Until then, oxlint's `tsgolint` covers type rules through the adapter codelaw already has.**

## 2 · The landscape, as read 2026-10-27 (sources §7)

| route | state | for codelaw |
|---|---|---|
| **oxlint + `tsgolint`** | stable since 2026-07-22; `oxlint-tsgolint` 7.0.2004 on TS 7.0.2; 60 of 62 targeted typescript-eslint rules; MIT; Go binary per platform (~22 MB); one-shot child process | best coverage today, but its stdin/stdout protocol is private and unversioned, and it reaches TS through `go:linkname` shims redone each TS bump |
| **TypeScript 7.1 API** | 7.0.2 GA 2026-07-08 with **no** API; 7.1 Beta 2026-10-20 adds a **preview** API (`typescript/sync`, `typescript/async`, snapshots) — JS calling the Go compiler; RC 2026-11-10, GA 2026-11-24 (iteration plan #63703); Apache-2.0 | the official route; no Go-library embedding exists — the API is JS-hosted, so a helper needs Node |
| typescript-eslint | no TS 7 support (peer `<6.1.0`); native-backend prototype PR #12803 targets 7.1 | nothing to reuse yet |
| Biome inference | v2.6; own Rust inference, a `types` domain; ~85% of `noFloatingPromises` cases; crates unstable | not a checker codelaw can embed |
| `tsrs` (Rust port of TS 7) | experimental, announced 2026-10-20 (Oxc/Cloudflare), Apache-2.0, no stable crate API | the reopen-trigger for a Rust-native route (§5) |
| Python: Pyrefly / ty | Pyrefly 1.4.0 stable (MIT), `pyrefly tsp` answers `getComputedType`; ty 0.0.91 beta | the later Python twin: Pyrefly over TSP |

## 3 · The rulings (2026-10-27, voice)

| # | fork | ruled | Arpit's words |
|---|---|---|---|
| R1 | oxlint's helper, or TypeScript's own engine | **TypeScript 7.1's own API**; the oxlint adapter fills the gap until it ships | *"wouldn't it be a good idea to go with TypeScript one since it's already close?"* · *"It does."* |
| R2 | a conflict between TypeScript's error and a codelaw finding | **reported once**: an adapter finding and a codelaw rule with the same `ported_from` dedupe as today; codelaw never re-checks what `ts/type-error` already reports | *"I'm good on the second one."* |
| R3 | a new LSP, or VS Code's TS server | **neither**: codelaw's own LSP stays; the helper is a backend behind it, the CLI, the daemon and the hooks. VS Code's `tsserver` belongs to the editor and is absent in CI, pre-commit and hooks | *"are we going to build out another LSP … or reuse the one that already exists in VS Code?"* |
| R4 | Node at runtime | **yes**, for type rules only | *"using Node at runtime makes sense. Let's go with it."* |
| R5 | where the rules live | **the existing `ts` pack** (`@codelaw/rules-ts`) | *"the rules go in TS pack, the existing one."* |
| R6 | pinned TypeScript, or the project's | **the project's installed copy**; ≥ 7.1 required, older → type rules skipped with a note; every report names the TS version used | *"go with the copy that project already uses … based on different version, there could be different rules."* |
| R7 | wait for 7.1 GA | **release early as preview**: the type rules ship `preview` (on only with `preview = true`) on the 7.1 RC; stable after 7.1 GA | *"we can go ahead and do a release earlier as well, and … say it is a preview."* |
| R8 | keeping up with TypeScript releases | **the recommendation**: a tested range (outside it: run with a warning, like adapters); a CI matrix — latest 7.1 blocking, `typescript@next` nightly, reporting only; a scheduled job like `fux-bump.yml` that opens a PR widening the range, or an issue when fixtures break | *"Go with the recommendation."* |

## 4 · How it works, and what is built

1. **The helper** — `codelaw-types`, codelaw's own JavaScript, shipped inside the engine packages (no new package). codelaw finds Node and the project's `typescript` (from the file up to the root), starts the helper with the `tsconfig`, and the helper loads the program once through `typescript/sync`. It answers a **fixed vocabulary of type questions** — promise-like, `any`, nullable, union members, `@deprecated`, `never`, thenable — over a versioned stdin/stdout protocol codelaw owns. Snapshots are disposed (the API leaks on the Go side otherwise).
2. **The `type:` clause** — rules stay declarative Markdown; a `type:` clause names the question; codelaw's fix engine builds the fixes (L4).
3. **TypeScript's own errors** — `ts/type-error` findings; TS code fixes as `suggest`.
4. **Warm, for L8** — CI and `check` load the program once; the daemon, LSP and `--watch` keep it warm (`snapshot.update`); the hook (`hook`, `hook --pre`) uses a warm program or skips type rules with a note, so `fix_one_file` and `hook_pre` hold.
5. **Rules first** — `no-floating-promises`, `no-misused-promises`, `await-thenable`, the `no-unsafe-*` family, `switch-exhaustiveness-check`; then the rest of the 2026-10-25 list (§6). Each carries `ported_from: [typescript-eslint/<rule>]`, so it dedupes against `adapter/oxlint/typescript/<rule>`.
6. **Reproducibility** — the report and `--version --json` name the TS version and the helper's protocol version; the cache key includes both.

## 5 · What it costs

| law / record | change | item |
|---|---|---|
| **L3** | type-aware checks move inside the core for TS when the project has TypeScript ≥ 7.1; decision 8's out-of-scope line and its consequence rewritten | W-104 |
| **L5** | the project's TypeScript version is an **input**, like the files: same files + same TS + same codelaw → byte-identical; no network, no model | W-104 |
| **L6** | Node at runtime for type rules only; the helper ships inside both engine packages; "complete alone" stays true for everything but type rules | W-104 |
| **L8** | hook paths skip cold type rules; new operations `check_types_cold` / `check_types_warm` start `unset` and are measured | W-104, W-105 |
| **L11** | **no new dependency** — the project's TypeScript is looked up like an adapter's binary, never installed by codelaw. **For the decider to confirm** at W-104 | W-104 |
| **L2** | TypeScript is Apache-2.0 — permissive | — |
| SR-ARCHITECTURE | "not decided yet: type-aware rules" becomes a new record, **SR-TYPES** | W-104 |
| SR-RULE-FORMAT | the `type:` clause | W-104 |

**Blockers:** none left that stop the work. TypeScript 7.1's API is a preview and can change between versions — W-106 is the answer. 7.1 GA (2026-11-24) gates only the move from preview to stable.

## 6 · The first proposal (2026-10-25), kept

**Arpit, 2026-10-25:** *"We need to build out type aware rules for sure … For type-aware rules, TypeScript Go … For now, put type aware rules and details in backlog. We will talk about it later."* It proposed a Go helper on typescript-go, shipped per platform as `@codelaw/types-<platform>`, with a pinned TS. W-101 replaced the Go helper with the project's own TS 7.1 (R1, R6), because the Go route means `go:linkname` shims and a binary per platform, and 7.1's API is official and a month out. Rules to port, from typescript-eslint `*-type-checked`: `no-floating-promises`, `no-misused-promises`, `await-thenable`, `require-await`, `return-await`, `switch-exhaustiveness-check`, `no-unnecessary-condition`, `no-unnecessary-type-assertion`, `prefer-nullish-coalescing`, `prefer-optional-chain`, `strict-boolean-expressions`, the five `no-unsafe-*`, `restrict-template-expressions`, `no-base-to-string`, `unbound-method`, `only-throw-error`, `prefer-promise-reject-errors`, `no-deprecated`, `consistent-type-exports`, `prefer-readonly`, `dot-notation`, `prefer-includes`.

**Reopen-trigger for another route:** `tsrs` publishes a stable Rust crate API — then a Rust-native checker, no Node, becomes possible; or 7.1's API is withdrawn.

## 7 · Sources (each as read 2026-10-27)

- oxc, *Type-Aware Linting Stable*, 2026-07-22 — https://oxc.rs/blog/2026-07-22-type-aware-linting-stable
- oxc, type-aware guide — https://oxc.rs/docs/guide/usage/linter/type-aware.html
- oxc-project/tsgolint — https://github.com/oxc-project/tsgolint · npm `oxlint-tsgolint` — https://www.npmjs.com/package/oxlint-tsgolint
- Microsoft, *Announcing TypeScript 7.1 Beta*, 2026-10-20 — https://devblogs.microsoft.com/typescript/announcing-typescript-7-1-beta/
- microsoft/typescript-go (closed; Go embedding discussion #481) — https://github.com/microsoft/typescript-go/discussions/481
- typescript-eslint PR #12803 — https://github.com/typescript-eslint/typescript-eslint/pull/12803
- Biome v2.5 — https://biomejs.dev/blog/biome-v2-5/ · Biome tsrs PR #12486 — https://github.com/biomejs/biome/pull/12486
- Cloudflare, *tsrs* — https://blog.cloudflare.com/typescript-rust-port
- Pyrefly releases — https://github.com/facebook/pyrefly/releases · Microsoft, *Announcing TSP* — https://devblogs.microsoft.com/python/announcing-tsp/
- ty releases — https://github.com/astral-sh/ty/releases
