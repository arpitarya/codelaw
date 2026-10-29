# @codelaw/rules-react · codelaw-rules-react

codelaw's React rules — one of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). The
same files ship on npm as `@codelaw/rules-react` and on PyPI as `codelaw-rules-react`.

```toml
# codelaw.toml
extends = ["codelaw:js/recommended", "codelaw:react/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `react/recommended` | 29 | eslint-plugin-react 7.37.5 `recommended` + `jsx-runtime` (17 of its 20 active rules); eslint-plugin-react-hooks 7.1.1's `rules-of-hooks`, `exhaustive-deps` and 8 of the 14 React Compiler rules its `recommended` turns on; and 2 of @eslint-react 5.28.8's hooks rules (`no-unnecessary-use-prefix`, the lazy-initialisation part of `use-state`), at `warn` as there |
| `react/conventions` | 2 | codelaw's starter catalogue: `react/hook-prefix`, `react/props-type-name` — opt in |
| `react/size` | 3 | Component size: lines per component, props per component and JSX nesting depth; opt in |

<!-- /docs-sync:presets -->

**In no preset:** `react/no-context` — no `createContext` or `useContext` — is a team's choice, so
turn it on under `[rules]`. Its page shows how to keep today's context as warnings while new
context fails, with a baseline.

Every rule is **reimplemented**, not vendored: the matchers, messages and docs
are written fresh, and each rule's `ported_from:` names the source rule.

## How the ports differ

Every rule matches its source's documented behaviour on the source's own
examples. Where codelaw differs, the rule's `## Differences from the source`
section says so. The larger ones:

| rule | difference |
|---|---|
| `react/jsx-key` | elements returned from a `.map()` callback's block body are not checked yet |
| `react/no-unknown-property` | renames HTML spellings (`class`, `onclick`, `tabindex`); in `.js`/`.jsx` the JavaScript grammar can't parse `class=` and `for=` |
| `react/exhaustive-deps` | dependencies compared by root name; setters, dispatchers and refs are stable |
| `react/rules-of-hooks` | an early `return` anywhere before a hook counts as making it conditional |
| the React Compiler rules (`no-set-state-in-render`, `no-set-state-in-effect`, `no-ref-in-render`, `purity`, `no-nested-components`, `use-memo`, `globals`, `error-boundaries`) | decided from the file's syntax and scopes: a setter, a ref or a global is recognised where it is declared, and nothing is followed through other functions or aliases, as the compiler's data flow does |

## Not ported, and why

| rule | why |
|---|---|
| `react/prop-types` | needs the component's declared prop types matched to its uses; TypeScript types do this job |
| `react/jsx-uses-vars` | marks JSX names as used for `no-unused-vars`; codelaw's semantic model already counts them |
| `react/react-in-jsx-scope`, `react/jsx-uses-react` | turned off by `jsx-runtime` |
| eslint-plugin-react-hooks' `immutability`, `preserve-manual-memoization` | need the compiler's alias and mutation analysis, or its own memoisation |
| eslint-plugin-react-hooks' `config`, `gating`, `incompatible-library`, `unsupported-syntax` | check the compiler's configuration, a list of libraries it can't memoise, and syntax it can't compile: about the React Compiler, not about the code's correctness |
| eslint-plugin-react-hooks' rules off in every preset (`capitalized-calls`, `memoized-effect-dependencies`, `exhaustive-effect-dependencies`, `no-deriving-state-in-effects`, `fbt`, `hooks`, `invariant`, `rule-suppression`, `syntax`, `todo`, `memo-dependencies`) | experimental or internal; `react/rules-of-hooks` and `react/exhaustive-deps` cover the two that check hooks |
| @eslint-react's `use-state` setter-naming and destructuring checks | naming conventions; the lazy-initialisation check is `react/prefer-lazy-state-init` |

## Data

`data/react/dom-property-aliases.json` — HTML attribute names and the React prop names that replace them, written for this pack.
