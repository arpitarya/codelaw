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
Every `*-type-checked` typescript-eslint config is out of scope: those rules
need type information (L3).
