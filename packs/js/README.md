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
| `react/recommended` | 19 | eslint-plugin-react 7.37.5 `recommended` + `jsx-runtime` (17 of its 20 active rules), and eslint-plugin-react-hooks 7.1.1's `rules-of-hooks` and `exhaustive-deps` |
| `node/recommended` | 10 | eslint-plugin-n 18.5.0 `recommended-module` — 10 of its 14 rules: the 3 that need no `package.json`, `node/hashbang`, which reads the nearest `package.json`'s `bin`, and the 6 import rules, which run on the project graph |
| `stencil/recommended` | 15 | @stencil/eslint-plugin 1.4.0 `recommended` — 15 of its 21 rules |
| `js/conventions` | 7 | codelaw's starter catalogue: `js/exports-at-end`, `js/import-groups`, `js/no-default-export`, `ts/type-only-imports`, `react/hook-prefix`, `react/props-type-name`, `stencil/tag-prefix` — opt in |

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
| `react/jsx-key` | elements returned from a `.map()` callback's block body are not checked yet |
| `react/no-unknown-property` | renames HTML spellings (`class`, `onclick`, `tabindex`); in `.js`/`.jsx` the JavaScript grammar can't parse `class=` and `for=` |
| `react/exhaustive-deps` | dependencies compared by root name; setters, dispatchers and refs are stable |
| `react/rules-of-hooks` | an early `return` anywhere before a hook counts as making it conditional |
| `node/no-deprecated-api` | the APIs come from the pack's `node/deprecated-api` table; the supported Node.js version is not read |
| `node/no-missing-import`, `node/no-missing-require` | `scope: project` rules on the project graph, which never reads `node_modules`: only the project's own paths are judged (relative, absolute, tsconfig `paths`), and only extensionless or JavaScript/TypeScript ones; a package name is not looked up |
| `node/no-extraneous-import`, `node/no-extraneous-require` | the nearest `package.json` only; a type-only import is satisfied by its `@types/` package |
| `node/no-unpublished-import`, `node/no-unpublished-require` | only in a package that is not `private` and lists `files`; `.npmignore` and the `convertPath` option are not read, so source files that build into `files` are not judged |
| `node/hashbang` | JavaScript and JSX only (no `convertPath` for TypeScript sources); `suggest`, since removing a hashbang can stop a script that is run directly |
| `stencil/tag-prefix` | with no `prefix` option set, a tag only needs some prefix |
| `stencil/prop-readonly` | the starter id for upstream `props-must-be-readonly` |

## Not ported, and why

| rule | why |
|---|---|
| `react/prop-types` | needs the component's declared prop types matched to its uses; TypeScript types do this job |
| `react/jsx-uses-vars` | marks JSX names as used for `no-unused-vars`; codelaw's semantic model already counts them |
| `react/react-in-jsx-scope`, `react/jsx-uses-react` | turned off by `jsx-runtime` |
| eslint-plugin-react-hooks' 14 React Compiler rules | need the React Compiler's analysis |
| `node/no-unsupported-features/es-builtins`, `/es-syntax`, `/node-builtins` | need the project's Node.js version from `package.json` and per-version feature tables |
| `node/process-exit-as-throw` | reports nothing; it only changes ESLint's code-path analysis |
| `stencil/async-methods`, `stencil/render-returns-host`, `stencil/single-export`, `stencil/strict-boolean-conditions` | need type information |
| `stencil/element-type` | needs the component's tag mapped to its element type name |
| `stencil/decorators-style` | layout: the formatter owns it |
| `stencil/dependency-suggestions` | advisory, with no fixed check to port |

Nothing from `js.configs.recommended` or typescript-eslint's non-type-checked
`recommended` is left out. Every `*-type-checked` typescript-eslint config is
out of scope: those rules need type information (L3).

## Data

`data/node/deprecated-api.json` — Node.js APIs the Node.js documentation deprecates, with their replacements, written for this pack.

`data/react/dom-property-aliases.json` — HTML attribute names and the React prop names that replace them, written for this pack.


`data/globals/*.json` — the globals of each environment `js/no-undef`'s `env`
option names, from the `globals` package 17.13.0 (MIT). See `NOTICE`.
