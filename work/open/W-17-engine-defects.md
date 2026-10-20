---
type: Handoff
name: W-17
description: "Arpit, 2026-10-20: the defects an independent read-only review of the engine found."
item: W-17
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-17 — Fix the engine defects found in the 2026-10-20 review

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Report a rule panic as `codelaw/internal-error` and exit 2 at the end; add the panic fixture | SR-ARCHITECTURE decision 25 kept (was B-026) | ✅ |
| 2 | Make a failed write fail the run | no 'fixed' when the disk is unchanged | ✅ |
| 3 | Fix the remaining findings below, each with a regression test | the engine does what its records say | ⏳ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: each finding is a correctness bug in the hot path; the fix and its regression test are judgment.

## Goal

Every finding below is fixed with a regression test, or recorded as a deliberate deviation in the owning record.

## Definition of done

| # | finding | where | severity | state |
|---|---|---|---|---|
| 1 | A rule panic is reported under the rule's own id with a note, never as `codelaw/internal-error`: `internal_errors()` is always 0, a `warn` rule that panics leaves `check` at exit 0, the result is cached, and `codelaw-ignore <rule>` hides it (SR-ARCHITECTURE decision 25). Add the panic fixture (was B-026). | `codelaw-engine/src/lint.rs:422` | 🧨 high | ✅ |
| 2 | A failed write is only a `warn` `codelaw/file-changed-during-run`, and the post-fix diagnostics are reported: a read-only file prints "fixed" and exits 0 with the disk unchanged. | `codelaw-engine/src/fix.rs:136 · codelaw-cli/src/main.rs:208` | 🧨 high | ✅ |
| 3 | `targets_at_least` is true when `target_version` is unset (always), so the `auto` fixes of `py/map-`, `py/zip-` and `py/batched-without-explicit-strict` add `strict=False`: `map(f, a, b, strict=False)` raises TypeError before Python 3.14 (reproduced 2026-10-20). Derive the target from `requires-python`, or make them `suggest` (L4: `auto` only where safe). | `codelaw-native/src/python_bugbear.rs:1103` | 🧨 high | ✅ |
| 4 | `js/no-useless-escape` (`auto`) has no tagged-template exemption: ``String.raw`\d+\.\d` `` loses its backslashes and the regex changes. | `codelaw-native/src/useless_escape.rs:28` | high | ✅ |
| 5 | `js/import-groups` (`auto`) reorders side-effect imports (`import './polyfills'`, CSS), changing what runs first. | `codelaw-native/src/conventions.rs:58` | high | ✅ |
| 6 | Nested configs are collected only during the walk, so files named on the command line (hook, pre-commit) ignore `pkg/codelaw.toml`: the same file gives different results. | `codelaw-engine/src/discover.rs:282` | major | ✅ |
| 7 | The fix verifier re-parses the host grammar only and compares error counts: no idempotence check, no "no new rule fires" (decision 5); a fix that breaks JS inside `<script>` or a fence passes. | `codelaw-fix/src/apply.rs:107` | major | ✅ |
| 8 | Walk errors (permission denied, symlink loop) are dropped by `.flatten()` and never reach `skipped[]` (decision 26): an unreadable subtree is silently unchecked. | `codelaw-engine/src/discover.rs:309` | major | ✅ |
| 9 | Every `package.json`/`pyproject.toml` met is parsed and a parse failure is a config error even without a `codelaw` section: one broken fixture file makes every run exit 2. | `codelaw-config/src/resolve.rs:211` | major | ✅ |
| 10 | `&other[..other.len().min(4)]` slices bytes: an unknown primitive name with a multi-byte character (`fixé`) panics at load — exit 101, not 2; a third-party pack can trigger it. | `codelaw-fix/src/primitives.rs` | major | ✅ |
| 11 | With `follow_symlinks = true`, `rename(tmp, link)` replaces the symlink with a regular file and leaves the target unchanged. | `codelaw-engine/src/fix.rs:106` | major (plausible) | ✅ |
| 12 | First-party status comes from the manifest name, so a third-party pack that copies a first-party name gets `native:` accepted. | `codelaw-rules/src/pack.rs` | minor | ✅ |
| 13 | The hash re-check runs before the temp file is written, not right before the rename; no fsync. | `codelaw-engine/src/fix.rs:100` | minor | ✅ |
| 14 | A semantic-model panic is swallowed with `.ok()`; `where:` on a `$$$` capture or a language with no model is accepted and is always false. | `codelaw-engine/src/lint.rs:438 · codelaw-match/src/lib.rs:191` | minor | ✅ |
| 15 | Suppressions aren't processed when no rule applies to a file; the marker matches anywhere in a comment (`// see codelaw-ignore docs` is an error). | `codelaw-engine/src/lint.rs:228 · suppress.rs:199` | minor | ✅ |
| 16 | `.parents(true)` with `require_git(false)` reads ignore files above the project root (e.g. `~/.gitignore`): a machine-dependent input (L5). | `codelaw-engine/src/discover.rs:304` | minor (plausible) | ✅ |
| 17 | An external fixer's stderr is never drained (a chatty fixer blocks), `join` can hang on a grandchild holding stdout, and only `sh` is killed on timeout. | `codelaw-fix/src/external.rs:107` | minor (plausible) | ✅ |
| 18 | `fix` doesn't dedupe like `check`; files outside the root print absolute paths; JSON edit offsets ignore a BOM (off by 3). | `codelaw-engine/src/fix.rs:169 · discover.rs:263` | minor | ✅ |
| 19 | `codelaw new-rule ../x` writes outside the rules folder. | `codelaw-cli/src/rules_cmd.rs:333` | minor | ✅ |
| 20 | `read_data` follows directory symlinks with no cycle check: a cycle in a pack's `data/` loops forever. | `codelaw-rules/src/pack.rs:265` | minor (plausible) | ✅ |
| 21 | Three `unnecessary_sort_by` warnings (`sort_by_key`) that `-D warnings` would fail on — hidden by W-18's masked clippy step. | `codelaw-fix (clippy)` | minor | ✅ |
| 22 | `md/link-fragments` slices `&t[i+1..i+3]` after `%`: `[x](#%aé)` panics. | `codelaw-native/src/data_md.rs:839` | major | ✅ |
| 23 | PEP 695 type parameters resolve in the wrong scope: `def first[T](xs: list[T]) -> T` reports `T` undefined (F821); class type params are invisible to methods and bases. | `codelaw-semantic/src/python.rs:120,138,365 · model.rs:374` | major | ✅ |
| 24 | A non-literal `__all__` (`["a"] + sub.__all__`, `.extend()`, `.append()`) becomes an empty export list, so F401 fires on re-exports. | `codelaw-semantic/src/python.rs:194,476` | major | ✅ |
| 25 | `export { Color }` marks only one namespace of a merged value+type name: `ts/no-unused-vars` flags the type half. | `codelaw-semantic/src/js.rs:609 · model.rs:376` | major | ✅ |
| 26 | `react/exhaustive-deps`' suggestion keys deps by root name: `[props.a, props.b]` collapses and the rewrite drops `props.a`. | `codelaw-native/src/react.rs:357` | minor | ✅ |
| 27 | `const [, setN] = useState(0)`: the setter is found by named-child position, so `setN` is reported missing. | `codelaw-native/src/react.rs:280` | minor | ✅ |
| 28 | `type Alias[T] = list[T]` never binds `Alias` (left side is a `generic_type`). | `codelaw-semantic/src/python.rs:334` | minor (plausible) | ✅ |
| 29 | Quote stripping slices bytes: an unterminated string ending in a multi-byte character can panic. | `codelaw-native/src/css_declarations.rs:331,438 · data_yaml.rs:26 · util.rs:96` | minor (plausible) | ✅ |
| 30 | `js/no-undef` ignores `/* global x */` and `/* eslint-env */`, which ESLint honours. | `codelaw-native/src/no_undef.rs:29` | minor | ✅ |
| 31 | `X: TypeAlias = "Decimal | int"` doesn't count as a use of a `TYPE_CHECKING` import (F401). | `codelaw-native/src/python_pyflakes.rs:151` | minor (plausible) | ✅ |
| 32 | Quadratic hook-reference scan and stray-end-tag search; per-element recursion (64 MB stacks mitigate). | `codelaw-native/src/react.rs:342 · html_dom.rs:461,491` | minor | ✅ |
| 33 | TS enum members are never declared and `NS.T` records `NS` as a value: false positives the day `js/no-undef` runs on TypeScript. | `codelaw-semantic (TS)` | latent | ✅ |
| 34 | `yaml/document-start` shares yamllint's rule name while the NOTICE says no names were taken: rename the rule or soften the claim (L2). | `packs/data/NOTICE:16` | minor | ✅ |

## Blockers

None.

## Record affected

[SR-ARCHITECTURE](../../records/0108_architecture.md) decisions 5, 7, 12, 18, 25, 26 · [SR-CONFIG](../../records/0110_config.md) · [SR-LINTER](../../records/0106_linter.md) decision 6.

## Hazards

- **Read-only review:** each finding was confirmed by reading the code; reproduce it as a failing test before fixing.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
