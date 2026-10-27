---
type: Handoff
name: W-105
description: "Build type-aware TS rules (W-101): the codelaw-types Node helper on the project's TypeScript 7.1 API, the type: clause, warm programs, ts/type-error and the first rules in the ts pack — preview on the 7.1 RC (2026-11-10), stable after 7.1 GA (2026-11-24)."
item: W-105
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-105 — Type-aware rules: helper, engine, first rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `codelaw-types` helper (codelaw's own JS) on `typescript/sync`: loads the `tsconfig` program once; codelaw's versioned stdin/stdout protocol; disposes snapshots | type answers | ⏳ |
| 2 | Engine: find Node and the project's `typescript`; < 7.1 or absent → type rules skipped with a note; the `type:` clause; cache key gains TS and protocol versions; report names both | rules can ask | ⏳ |
| 3 | Warm: daemon, LSP and `--watch` keep the program (`snapshot.update`); `hook` / `hook --pre` use it warm or skip with a note | L8 holds | ⏳ |
| 4 | `ts/type-error` (TS diagnostics; TS code fixes as `suggest`) | TypeScript's own errors, once | ⏳ |
| 5 | First rules in `packs/ts`, `preview`, `ported_from` set: `no-floating-promises`, `no-misused-promises`, `await-thenable`, the `no-unsafe-*` family, `switch-exhaustiveness-check`; fixtures | the important ones | ⏳ |
| 6 | **Release as preview on the 7.1 RC (2026-11-10)**; measure `check_types_*` on CI | early release | ⏳ |
| 7 | After 7.1 GA (2026-11-24): re-run on GA, then the rules go to promotion (W-100's criterion); port the rest of the list | stable | ⏳ |

**Arpit, 2026-10-27 (voice), W-101:** rulings R1–R8 in [the proposal](../proposals/type-aware-rules.md) §3 — TypeScript 7.1's own API through the project's installed copy, Node at runtime, rules in the `ts` pack, preview on the 7.1 RC, *"Go with the recommendation"* on version checks.

**Model:** Opus — a new process boundary and a protocol.

Built against 7.1 Beta (2026-10-20) until the RC lands.

## Definition of done

- Steps 1–6 done, with a fixture repo checked under 7.1 with each first rule firing and deduping against `adapter/oxlint/typescript/*`; step 7 may close as its own item.

## Record affected

SR-TYPES (W-104), SR-RULE-PACKS (the `ts` pack's README drops "out of scope" for `*-type-checked`).

## Hazards

- The 7.1 API is a preview: keep every call behind the helper so a break is one file.
- Forgetting `dispose` leaks on the Go side — test a long `--watch` run for memory.
