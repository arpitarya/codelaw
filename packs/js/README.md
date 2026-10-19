# @codelaw/rules-js · codelaw-rules-js

codelaw's rules for JavaScript, TypeScript, JSX and TSX — one of the five
first-party rule packs ([SR-RULE-PACKS](../../records/0101_rule-packs.md)). The
same files ship on npm as `@codelaw/rules-js` and on PyPI as
`codelaw-rules-js`.

```toml
# codelaw.toml
extends = ["codelaw:js/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

| preset | rules | ported from |
|---|---|---|
| `js/recommended` | 64 | ESLint `js.configs.recommended`, `@eslint/js` 10.0.1 — all 64 rules |
| `ts/recommended` | 24 | typescript-eslint `recommended` 8.71.2 (not type-checked) — all 20 rules — plus the four core rules its `eslint-recommended` switches on for TypeScript: `js/no-var`, `js/prefer-const`, `js/prefer-rest-params`, `js/prefer-spread` |

Rules that typescript-eslint's `eslint-recommended` switches off for TypeScript
(`no-undef`, `no-redeclare`, `no-dupe-class-members` and the rest, which the
compiler checks) declare only `javascript` and `jsx` here. The four core rules
`ts/recommended` adds declare all four languages, so they also run on
JavaScript files when that preset is on.

Every rule is **reimplemented**, not vendored: the matchers, messages and docs
are written fresh, and each rule's `ported_from:` names the source rule.

## How the ports differ

Every rule matches its source's documented behaviour on the source's own
examples. Where codelaw differs, the rule's `## Differences from the source`
section says so. The larger ones:

| rule | difference |
|---|---|
| `js/constructor-super`, `js/no-this-before-super` | follow the constructor's statements in straight-line order; a `super()` on any branch counts |
| `js/no-constant-condition` | literal conditions only; ESLint also folds `typeof x`, `!1` and similar |
| `js/no-extra-boolean-cast` | `!!x` in a condition; `Boolean(x)` in a condition is not checked yet |
| `js/no-invalid-regexp` | structural problems only (brackets, groups, quantifiers, flags) |
| `js/no-useless-assignment` | dead stores a straight-line reading proves; ESLint follows every code path |
| `js/no-useless-escape` | strings and templates; escapes inside regular expressions are not checked yet |
| `js/no-undef` | environments come from the `env` option (`shared-node-browser` by default), using the `globals` data tables |
| `ts/no-unused-expressions` | `x satisfies T;` is a compile-time type test and is allowed |
| `js/prefer-spread` | only `f.apply(null, args)` on a plain function name |
| `js/no-var` | `suggest`, not `auto`: `let` is block-scoped, so the swap can change meaning |

## Not ported, and why

Nothing from `js.configs.recommended` or typescript-eslint's non-type-checked
`recommended` is left out. Every `*-type-checked` typescript-eslint config is
out of scope: those rules need type information (L3).

## Data

`data/globals/*.json` — the globals of each environment `js/no-undef`'s `env`
option names, from the `globals` package 17.13.0 (MIT). See `NOTICE`.
