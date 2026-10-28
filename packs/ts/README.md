# @codelaw/rules-ts

codelaw's TypeScript rules — one of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). It ships on
npm only, as `@codelaw/rules-ts`; PyPI has no copy (L6 decision 11).

```toml
# codelaw.toml
extends = ["codelaw:js/recommended", "codelaw:js/typescript", "codelaw:ts/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `ts/recommended` | 20 | typescript-eslint `recommended` 8.71.2 (not type-checked) — all 20 rules; the four core rules its `eslint-recommended` switches on are `codelaw:js/typescript` in `@codelaw/rules-js` |
| `ts/conventions` | 1 | codelaw's starter catalogue: `ts/type-only-imports` — opt in |
| `ts/type-checked` | 6 | typescript-eslint's type-checked rules, answered by the project's own TypeScript ≥ 7.1 (SR-TYPES); every rule is preview until TypeScript 7.1 is GA |

<!-- /docs-sync:presets -->

Every rule is **reimplemented**, not vendored: the matchers, messages and docs
are written fresh, and each rule's `ported_from:` names the source rule.

## How the ports differ

Every rule matches its source's documented behaviour on the source's own
examples. Where codelaw differs, the rule's `## Differences from the source`
section says so. The larger ones:

| rule | difference |
|---|---|
| `ts/no-unused-expressions` | `x satisfies T;` is a compile-time type test and is allowed |

Nothing from typescript-eslint's non-type-checked `recommended` is left out.

## Type-checked rules

`ts/type-checked` holds the rules that need types, answered by your project's
own TypeScript — 7.1 or later, the `typescript` package installed from the
file's folder up to the project root — through a small Node helper inside
codelaw ([SR-TYPES](../../records/0117_types.md)). codelaw installs no
TypeScript. Without Node or TypeScript ≥ 7.1 they are skipped, and the report
says why; everything else runs. Every one is `preview` until TypeScript 7.1 is
GA, and the report names the TypeScript version that answered.

```toml
# codelaw.toml
extends = ["codelaw:ts/recommended", "codelaw:ts/type-checked"]
preview = true
```

| rule | from typescript-eslint |
|---|---|
| `ts/no-floating-promises` | `no-floating-promises` |
| `ts/await-thenable` | `await-thenable` |
| `ts/no-unsafe-call` | `no-unsafe-call` |
| `ts/no-unsafe-member-access` | `no-unsafe-member-access` |
| `ts/no-unsafe-return` | `no-unsafe-return` |
| `ts/type-error` | — TypeScript's own errors, as `tsc` reports them |

The rest of typescript-eslint's `recommended-type-checked` is not built yet
(W-105). `no-misused-promises`' condition check is TypeScript's own error
TS2801, which `ts/type-error` reports. A hook (`codelaw hook`) never loads a
type program: it uses a warm one (`codelaw daemon start`) or skips the type
rules with a note.
