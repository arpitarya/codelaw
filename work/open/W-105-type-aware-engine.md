---
type: Handoff
name: W-105
description: "Build type-aware TS rules (W-101): the codelaw-types Node helper on the project's TypeScript 7.1 API, the type: clause, warm programs, ts/type-error and the first rules in the ts pack — preview on the 7.1 RC (2026-11-10), stable after 7.1 GA (2026-11-24)."
item: W-105
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Claude Code (Opus 5.5), W-105 first landing"
ball: agent
---

# W-105 — Type-aware rules: helper, engine, first rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `codelaw-types` helper (codelaw's own JS) on `typescript/sync`: loads the `tsconfig` program once; codelaw's versioned stdin/stdout protocol; disposes snapshots | type answers | ✅ 2026-10-28 — `crates/codelaw-types` (`helper.mjs` run as `node --input-type=module --eval`, protocol 1: `hello`/`check`/`changed`/`refresh`/`close`, UTF-8↔UTF-16 offsets, an overlay snapshot for unsaved text, every snapshot disposed) |
| 2 | Engine: find Node and the project's `typescript`; < 7.1 or absent → type rules skipped with a note; the `type:` clause; cache key gains TS and protocol versions; report names both | rules can ask | ✅ 2026-10-28 — discovery from the file's folder up (`CODELAW_TYPESCRIPT` as the tester's fallback), the ≥ 7.1 gate and the tested range `>=7.1.0-beta, <7.2.0`, the `type:` clause (SR-RULE-FORMAT 3f), `--format json` `types {protocol, typescript[], notes[]}` and the summary line; a file a type rule checks is **not cached** while a program answers (its imports are outside any per-file key — SR-TYPES decision 12 amended) · ⏳ `codelaw --version --json` does not exist yet |
| 3 | Warm: daemon, LSP and `--watch` keep the program (`snapshot.update`); `hook` / `hook --pre` use it warm or skip with a note | L8 holds | ✅ 2026-10-28 — helpers kept per process (≤ 4), `refresh` re-reads files whose mtime moved, `--watch` sends `changed` and re-checks every typed file; `hook`/`hook --pre` run with `no_cold_types` (`crates/codelaw-cli/tests/types.rs` `a_hook_uses_a_warm_program_and_never_a_cold_one`) · ⏳ the long `--watch` memory test (Hazards) |
| 4 | `ts/type-error` (TS diagnostics; TS code fixes as `suggest`) | TypeScript's own errors, once | ✅ 2026-10-28 — `native: ts-type-error`, syntactic + semantic errors and warnings as `TS<code>: …`; `manual`, since 7.1's API has no code fixes |
| 5 | First rules in `packs/ts`, `preview`, `ported_from` set: `no-floating-promises`, `no-misused-promises`, `await-thenable`, the `no-unsafe-*` family, `switch-exhaustiveness-check`; fixtures | the important ones | 🟡 2026-10-28 — `ts/type-checked`: `no-floating-promises`, `await-thenable`, `no-unsafe-call`, `no-unsafe-member-access`, `no-unsafe-return`, `type-error`, each passing `codelaw test` under 7.1.0-beta; `fixtures/types` fires each once and an oxlint port is shown once. `no-misused-promises`' condition check is TS2801 (`ts/type-error`), so not built (SR-TYPES decision 10) · ⏳ see *Left* |
| 6 | **Release as preview on the 7.1 RC (2026-11-10)**; measure `check_types_*` on CI | early release | ⏳ the RC is not out (today 2026-10-28) |
| 7 | After 7.1 GA (2026-11-24): re-run on GA, then the rules go to promotion (W-100's criterion); port the rest of the list | stable | ⏳ |

### Left (exact)

1. `ts/switch-exhaustiveness-check` — needs a new type question (which union members a `switch` leaves out): an amendment of SR-TYPES decision 4.
2. The rest of the `no-unsafe-*` family: `no-unsafe-argument`, `no-unsafe-assignment`; and `no-misused-promises`' `checksVoidReturn` (an async function where a `void` callback is expected) — each needs a question the vocabulary lacks.
3. `ts/await-thenable` reports `await` on a value typed `unknown`, which typescript-eslint allows: an `unknown` question (an SR-TYPES decision 4 amendment) would fix it — **for the decider**.
4. `codelaw --version --json` naming the TypeScript and the protocol (SR-TYPES decision 12; L5).
5. L8: `codelaw bench` operations `check_types_cold` / `check_types_warm` on a TypeScript workload, Node and the fixture's TypeScript in the `perf` job, and their `unset` rows in `perf/budgets.toml` (SR-LAW-8 §Measurements says they enter with the bench). Local, unloaded-Mac context only: `fixtures/types` (5 TS files) checks cold in ≈ 0.7 s wall including Node and the program load.
6. The type rules' corpus counts: the `corpus` job has no TypeScript, so `packs/ts/corpus.toml` holds 0 for each (the tester notes it).
7. A long `--watch` run's memory, for the Go-side leak (Hazards).
8. Step 6: re-pin `fixtures/types` to `7.1.0-rc` on 2026-11-10, rerun, release as preview; step 7 after GA.

**Arpit, 2026-10-27 (voice), W-101:** rulings R1–R8 in [the proposal](../proposals/type-aware-rules.md) §3 — TypeScript 7.1's own API through the project's installed copy, Node at runtime, rules in the `ts` pack, preview on the 7.1 RC, *"Go with the recommendation"* on version checks.

**Model:** Opus — a new process boundary and a protocol.

Built against 7.1 Beta (2026-10-20) until the RC lands. First landing 2026-10-28 (Claude Code): steps 1–4 and five first rules; the item stays open on *Left* above.

## Definition of done

- Steps 1–6 done, with a fixture repo checked under 7.1 with each first rule firing and deduping against `adapter/oxlint/typescript/*`; step 7 may close as its own item.

## Record affected

SR-TYPES (W-104; amended with the as-built decisions 2–5, 7–10 and 12, and its Ownership rows, 2026-10-28), SR-RULE-FORMAT 3f, SR-RULE-PACKS (the `ts` row names `ts/type-checked`; the pack's README drops "out of scope" for `*-type-checked`), SR-PACKAGES decision 1 (`codelaw-types` in the engines layer), L11 §3 D-45 (the pinned tree).

## Hazards

- The 7.1 API is a preview: keep every call behind the helper so a break is one file.
- Forgetting `dispose` leaks on the Go side — test a long `--watch` run for memory.
