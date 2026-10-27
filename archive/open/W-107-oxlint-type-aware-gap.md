---
type: Handoff
name: W-107
description: "Type rules today, until W-105 ships (W-101 R1): prove and document that oxlint's type-aware mode (oxlint-tsgolint 7.x, options.typeAware in .oxlintrc.json) flows through codelaw's existing oxlint adapter, typescript/* rule ids included."
item: W-107
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5)"
ball: closed
---

# W-107 — The gap filler: oxlint type-aware through the adapter

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | A fixture with `oxlint-tsgolint` 7.x and `options.typeAware: true` in `.oxlintrc.json`; check the adapter's parser reads `typescript/no-floating-promises` and friends | type rules work today | ✅ `crates/codelaw-adapters/tests/fixtures/oxlint-type-aware/` (built from oxlint's documented JSON, not captured: L11) and `oxlint_type_aware_findings_keep_their_typescript_ids`; through the binary, `oxlint_type_aware_findings_pass_through_and_are_not_cached` (`crates/codelaw-cli/tests/adapters.rs`) |
| 2 | Check whether oxlint's `--format=json` carries fix edits for them (unverified in W-101's research); if not, they stay `manual` with docs links, as today | the fix class, named | ✅ no edits: oxlint's JSON formatter (`apps/oxlint/src/output_formatter/json.rs`) renders miette's report (message, code, severity, url, help, filename, labels), with no fix field, so they stay `manual` with oxlint's help and the oxc.rs docs link |
| 3 | USAGE: "type-aware rules today" — install, the one config line, and that codelaw's own rules replace it via `ported_from` once W-105 ships | people know how | ✅ USAGE *Type rules today* |

**Arpit, 2026-10-27 (voice), W-101:** rulings R1–R8 in [the proposal](../../work/proposals/type-aware-rules.md) §3 — TypeScript 7.1's own API through the project's installed copy, Node at runtime, rules in the `ts` pack, preview on the 7.1 RC, *"Go with the recommendation"* on version checks.

**Model:** Sonnet — a fixture and a doc on an existing adapter.

## Definition of done

- The fixture runs in `crates/codelaw-adapters/tests`; USAGE has the section.

## Record affected

SR-LINTER decision 2 (the type-aware note; `ported_plugins` in *Shown once*) and SR-ARCHITECTURE decision 19 (`uncached_when` in the cache key line). The tested range is unchanged.

## Hazards

- `tsgolint` needs TypeScript 7.0+ in the fixture; legacy `baseUrl` is unsupported.

## Outcome (2026-10-27, Claude Code)

- **It flows through with no change to the parser.** oxlint's type-aware findings arrive in the same `--format=json` with codes like `typescript(no-floating-promises)` (oxc's own tsgolint snapshots), so they become `adapter/oxlint/typescript/<rule>` with oxlint's severity and byte span, `manual`. The adapter runs the project's own `node_modules/.bin/oxlint`, which reads the user's `.oxlintrc.json`, so codelaw needs no flag.
- **Two adapter fixes it showed:**
  - *Shown once was broken for every `typescript/*` rule.* oxlint names typescript-eslint's rules `typescript/<rule>`; the `ts` pack's `ported_from:` names them `typescript-eslint/<rule>` (21 rules today, e.g. `ts/ban-ts-comment`), so both findings were shown. The manifest's new `ported_plugins = { typescript = "typescript-eslint" }` maps them — and W-105's rules dedupe the same way.
  - *A type-aware result was cached per file.* A type rule's verdict depends on the files a file imports, which no per-file key covers, so a stale clean result could be served. The manifest's new `uncached_when = "typeAware"`: while the config governing a file names `typeAware`, oxlint's results are never cached (SR-ARCHITECTURE decision 19).
- **Not proven live:** no oxlint, `oxlint-tsgolint` or TypeScript 7 is installed in this repo or CI, and none is ratified (L11; W-99's D-40 is proposed for the bench only). The live capture is in the inbox: run the command in the test's doc comment, and an agent swaps the captured `stdout.json` in. The tested range (`>=1.89.0, <2.0.0`) is unchanged.
- **Left, not this item's:** oxlint's `react(...)`, `react-hooks` rules (under `react`) and `node(...)` ids do not match the packs' `eslint-plugin-react/`, `eslint-plugin-react-hooks/` and `eslint-plugin-n/` `ported_from:` names either; only `typescript` was verified against oxc's output, so only it is mapped.
