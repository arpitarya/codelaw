---
type: Handoff
name: W-105
description: "Type-aware TypeScript (W-101): the codelaw-types helper, the type: clause, ts/type-error and the ts rules — preview on the 7.1 RC (2026-11-10), stable after GA (2026-11-24) — and the TypeScript version watch (was W-106). Merged 2026-10-30."
item: W-105
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-105 — Type-aware TypeScript, and its version watch

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `codelaw-types` helper (codelaw's own JS) on `typescript/sync`: loads the `tsconfig` program once; codelaw's versioned stdin/stdout protocol; disposes snapshots | type answers | ✅ 2026-10-28 — `crates/codelaw-types` (`helper.mjs` run as `node --input-type=module --eval`, protocol 1: `hello`/`check`/`changed`/`refresh`/`close`, UTF-8↔UTF-16 offsets, an overlay snapshot for unsaved text, every snapshot disposed) |
| 2 | Engine: find Node and the project's `typescript`; < 7.1 or absent → type rules skipped with a note; the `type:` clause; cache key gains TS and protocol versions; report names both | rules can ask | ✅ 2026-10-28 — discovery from the file's folder up (`CODELAW_TYPESCRIPT` as the tester's fallback), the ≥ 7.1 gate and the tested range `>=7.1.0-beta, <7.2.0`, the `type:` clause (SR-RULE-FORMAT 3f), `--format json` `types {protocol, typescript[], notes[]}` and the summary line; a file a type rule checks is **not cached** while a program answers (its imports are outside any per-file key — SR-TYPES decision 12 amended) · ✅ 2026-10-28 `codelaw --version --json`: `codelaw`, `types {protocol, tested, typescript {version, path, api, tested} \| null, node}` from the current folder, nothing started (`codelaw_engine::types_version`; `tests/types.rs` `version_json_names_the_protocol_and_the_project_typescript`; SR-CLI-MCP decision 1) |
| 3 | Warm: daemon, LSP and `--watch` keep the program (`snapshot.update`); `hook` / `hook --pre` use it warm or skip with a note | L8 holds | ✅ 2026-10-28 — helpers kept per process (≤ 4), `refresh` re-reads files whose mtime moved, `--watch` sends `changed` and re-checks every typed file; `hook`/`hook --pre` run with `no_cold_types` (`crates/codelaw-cli/tests/types.rs` `a_hook_uses_a_warm_program_and_never_a_cold_one`) · ✅ 2026-10-28 the long `--watch` memory test: `a_long_watch_does_not_grow_with_its_rechecks`, 100 re-checks, the watch and its helper within 15 % of their size after 20 (139 → 146 MiB on macOS; with `replaceBase`'s `dispose` removed, 144 → 191 MiB and the test fails) |
| 4 | `ts/type-error` (TS diagnostics; TS code fixes as `suggest`) | TypeScript's own errors, once | ✅ 2026-10-28 — `native: ts-type-error`, syntactic + semantic errors and warnings as `TS<code>: …`; `manual`, since 7.1's API has no code fixes |
| 5 | First rules in `packs/ts`, `preview`, `ported_from` set: `no-floating-promises`, `no-misused-promises`, `await-thenable`, the `no-unsafe-*` family, `switch-exhaustiveness-check`; fixtures | the important ones | 🟡 2026-10-28 — `ts/type-checked`: `no-floating-promises`, `await-thenable`, `no-unsafe-call`, `no-unsafe-member-access`, `no-unsafe-return`, `type-error`, each passing `codelaw test` under 7.1.0-beta; `fixtures/types` fires each once and an oxlint port is shown once. `no-misused-promises`' condition check is TS2801 (`ts/type-error`), so not built (SR-TYPES decision 10) · ⏳ see *Left* |
| 6 | **Release as preview on the 7.1 RC (2026-11-10)**; measure `check_types_*` on CI | early release | 🟡 2026-10-28 — `codelaw bench --typescript <dir>` times `check_types_cold` / `check_types_warm` on the type workload (the reference workload's 2,200 `.ts` files, one `tsconfig.json`, `ts/type-checked` only; digest `5341cd61a21e66c7`), the `perf` job installs `fixtures/types`' TypeScript and passes it, and both are `unset` in `perf/budgets.toml` with a `proposed` from one loaded-Mac run (cold p95 3955 ms, warm 3170 ms) · ⏳ the RC is not out (today 2026-10-28) |
| 7 | After 7.1 GA (2026-11-24): re-run on GA, then the rules go to promotion (W-100's criterion); port the rest of the list | stable | ⏳ |
| W-106 | TypeScript version watch — merged in; its steps under [§ W-106](#merged-in-w-106) | one item, not two | ⏳ — see § |

**Merged 2026-10-30 (Arpit, text):** *"There are too many work items. See if we can review all the work items and merge them into few of them."* → *"go ahead, merge them."* Merged in: W-106.

**Arpit, 2026-10-29 (voice docket; written on *"file it."*):** asked whether agents may extend SR-TYPES decision 4's vocabulary for the four rules (exhaustive `switch`, `any` into a typed target, a promise where a `void` callback is expected, `await` on `unknown`): *"Add it."* Then, recommended to build the four as previews and to profile the slowest type rule before any budget is put to him (he said on W-104 *"make sure it is as fast as possible"*): *"Let's go ahead and build it out."*

### Left (exact)

1. **For the decider** (inbox row *Type questions*): may an agent extend SR-TYPES decision 4's vocabulary? Items 2–4 each need a question it lacks.
2. `ts/switch-exhaustiveness-check` — which union members a `switch` leaves out.
3. The rest of the `no-unsafe-*` family, `no-unsafe-argument` and `no-unsafe-assignment` — an `any` value passed or assigned where the target's type is not `any`; and `no-misused-promises`' `checksVoidReturn` — a Promise-returning function where a `void`-returning callback is expected.
4. `ts/await-thenable` reports `await` on a value typed `unknown`, which typescript-eslint allows — an `unknown` question.
5. L8: after three `perf` runs on `main` that time `check_types_cold` / `check_types_warm`, an inbox row to the decider with decision 4's method applied (min(target, 1.5 × the worst p95); no design target exists for either). Context: on the loaded Mac the warm run is only 20 % under the cold one — the 2,200 files' questions go one at a time through one helper, which a later change could batch.
6. The type rules' corpus counts stay 0, and installing TypeScript in the `corpus` job would not make them real: level 4 checks each corpus file alone (the rule tester's root is not the repo), and the corpus repos have no `node_modules`, so every import reads as `any`. Tried 2026-10-28 with 7.1.0-beta over the `ts` pack's four repos: `no-unsafe-call` 65 665, `no-unsafe-member-access` 42 581, `no-unsafe-return` 2 022, `type-error` 23 377, `no-floating-promises` 141, `await-thenable` 0 — the unresolved imports, not the code. Real counts need level 4 to check a typed rule's files in their repo's program (codelaw-test, SR-RULE-AUTHORING) **and** each corpus repo's dependencies installed in CI, which L11 would have to ratify — not asked yet.
7. Step 6: re-pin `fixtures/types` to `7.1.0-rc` on 2026-11-10, rerun, release as preview; step 7 after GA.

**Arpit, 2026-10-27 (voice), W-101:** rulings R1–R8 in [the proposal](../proposals/type-aware-rules.md) §3 — TypeScript 7.1's own API through the project's installed copy, Node at runtime, rules in the `ts` pack, preview on the 7.1 RC, *"Go with the recommendation"* on version checks.

**Model:** Opus — a new process boundary and a protocol.

Built against 7.1 Beta (2026-10-20) until the RC lands. First landing 2026-10-28 (Claude Code): steps 1–4 and five first rules. Second landing 2026-10-28 (Claude Code): `--version --json`, the `check_types_*` bench and its CI wiring, the long-`--watch` memory test; the corpus counts tried and found not real. The item stays open on *Left* above, its next step the decider's.

## Definition of done

- Steps 1–6 done, with a fixture repo checked under 7.1 with each first rule firing and deduping against `adapter/oxlint/typescript/*`; step 7 may close as its own item.

## Record affected

SR-TYPES (W-104; amended with the as-built decisions 2–5, 7–10 and 12, and its Ownership rows, 2026-10-28), SR-RULE-FORMAT 3f, SR-RULE-PACKS (the `ts` row names `ts/type-checked`; the pack's README drops "out of scope" for `*-type-checked`), SR-PACKAGES decision 1 (`codelaw-types` in the engines layer), L11 §3 D-45 (the pinned tree). Second landing: SR-CLI-MCP decisions 1 and 10 (`--version --json`, the type workload), SR-TYPES decisions 2 and 12 and §Consequences, L8 §How it is enforced and §Consequences (status lines only), SR-PACKAGES (the `perf` job's TypeScript), SR-ARCHITECTURE (`types_version`).

## Hazards

- The 7.1 API is a preview: keep every call behind the helper so a break is one file.
- Forgetting `dispose` leaks on the Go side — test a long `--watch` run for memory (done: `a_long_watch_does_not_grow_with_its_rechecks`).

## Merged in: W-106

<a id="merged-in-w-106"></a>

**W-106 — TypeScript version watch**, merged into W-105 on 2026-10-30; its file is archived at [`archive/open/W-106-typescript-version-watch.md`](../../archive/open/W-106-typescript-version-watch.md). As it stood:

### At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | A tested TS range in SR-TYPES and the helper; outside it, type rules run with a warning (the adapter pattern) | users on newer TS keep working | ⏳ |
| 2 | `ci.yml`: the type-rule fixtures against the latest 7.1 on every change (blocking) | no regressions | ⏳ |
| 3 | Nightly: the same against `typescript@next`, report only | early warning | ⏳ |
| 4 | `typescript-bump.yml`, modelled on `fux-bump.yml`: a new TS release → run the fixtures → PR widening the range, or an issue naming what broke | nobody has to watch | ⏳ |

**Arpit, 2026-10-27 (voice), W-101:** rulings R1–R8 in [the proposal](../proposals/type-aware-rules.md) §3 — TypeScript 7.1's own API through the project's installed copy, Node at runtime, rules in the `ts` pack, preview on the 7.1 RC, *"Go with the recommendation"* on version checks.

**Model:** Sonnet — CI wiring on a known pattern.

### Definition of done

- A dispatch with an older and a newer TS version each produces the right PR or issue (as W-91 proved `fux-bump.yml`).

### Record affected

SR-TYPES (W-104) for the range; SR-PACKAGES if `ci.yml` jobs are named there.

### Hazards

- The nightly must never block `main`.
