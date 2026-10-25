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
| `react/recommended` | 19 | eslint-plugin-react 7.37.5 `recommended` + `jsx-runtime` (17 of its 20 active rules), and eslint-plugin-react-hooks 7.1.1's `rules-of-hooks` and `exhaustive-deps` |
| `react/conventions` | 2 | codelaw's starter catalogue: `react/hook-prefix`, `react/props-type-name` — opt in |

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

## Not ported, and why

| rule | why |
|---|---|
| `react/prop-types` | needs the component's declared prop types matched to its uses; TypeScript types do this job |
| `react/jsx-uses-vars` | marks JSX names as used for `no-unused-vars`; codelaw's semantic model already counts them |
| `react/react-in-jsx-scope`, `react/jsx-uses-react` | turned off by `jsx-runtime` |
| eslint-plugin-react-hooks' 14 React Compiler rules | need the React Compiler's analysis |

## Data

`data/react/dom-property-aliases.json` — HTML attribute names and the React prop names that replace them, written for this pack.
