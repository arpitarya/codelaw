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

## Not ported, and why

Nothing from `js.configs.recommended` is left out.

## Data

`data/globals/*.json` — the globals of each environment `js/no-undef`'s `env`
option names, from the `globals` package 17.13.0 (MIT). See `NOTICE`.
