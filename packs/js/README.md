# @codelaw/rules-js · codelaw-rules-js

codelaw's core JavaScript rules, which also run on TypeScript, JSX and TSX — one of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). The
same files ship on npm as `@codelaw/rules-js` and on PyPI as `codelaw-rules-js`.

```toml
# codelaw.toml
extends = ["codelaw:js/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `js/recommended` | 66 | ESLint `js.configs.recommended`, `@eslint/js` 10.0.1 — all 64 rules — plus `js/no-console` at `warn`, which ESLint leaves out, and `js/use-instead` at `warn`, inert until `replacements` is set (W-73); a team turns it to `error` in `[profiles.ci]` |
| `js/typescript` | 4 | the four core rules typescript-eslint 8.71.2's `eslint-recommended` switches on for TypeScript: `js/no-var`, `js/prefer-const`, `js/prefer-rest-params`, `js/prefer-spread`; use it with `codelaw:ts/recommended` |
| `js/conventions` | 3 | codelaw's starter catalogue: `js/exports-at-end`, `js/import-groups`, `js/no-default-export` — opt in |
| `js/project` | 1 | `graph/no-unused-exports`, the rule that reads the whole repo, from knip's `exports` and `types` — opt in |
| `js/size` | 3 | size limits at `warn`, with ESLint's defaults: `js/max-lines` (300 per file), `js/max-lines-per-function` (50) and `js/max-params` (3) — opt in |

<!-- /docs-sync:presets -->

Rules that typescript-eslint's `eslint-recommended` switches off for TypeScript
(`no-undef`, `no-redeclare`, `no-dupe-class-members` and the rest, which the
compiler checks) declare only `javascript` and `jsx` here. The four rules in
`js/typescript` declare all four languages, so they also run on JavaScript
files when that preset is on.

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
| `js/prefer-spread` | only `f.apply(null, args)` on a plain function name |
| `js/no-var` | `suggest`, not `auto`: `let` is block-scoped, so the swap can change meaning |

Nothing from `js.configs.recommended` is left out.

## Data

`data/globals/*.json` — the globals of each environment `js/no-undef`'s `env`
option names, from the `globals` package 17.13.0 (MIT). See `NOTICE`.
