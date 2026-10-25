# @codelaw/rules-tanstack-query

codelaw's TanStack Query rules — one of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). It ships on npm only, as
`@codelaw/rules-tanstack-query`: TanStack Query is a JavaScript library.

```toml
# codelaw.toml
extends = ["codelaw:js/recommended", "codelaw:react/recommended", "codelaw:tanstack-query/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

`codelaw init` adds it when `package.json` depends on `@tanstack/react-query`
or `react-query`.

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `tanstack-query/recommended` | 7 | @tanstack/eslint-plugin-query 5.104.5 `recommended` (all 7), at its severities |

<!-- /docs-sync:presets -->

Every rule is **reimplemented**, not vendored: the matchers, messages and docs
are written fresh, and each rule's `ported_from:` names the source rule. The
rules read the option objects TanStack Query's hooks take, on codelaw's
semantic model (`native:` rules, like `react/exhaustive-deps`). A rule that
follows a hook (`no-rest-destructuring`, `no-unstable-deps`,
`stable-query-client`) checks that it is imported from a `@tanstack/*-query`
package, as upstream does.

| rule | fix | checks |
|---|---|---|
| `tanstack-query/exhaustive-deps` | suggest | the query key names every value the query function reads |
| `tanstack-query/stable-query-client` | suggest | `new QueryClient()` is not made on every render |
| `tanstack-query/no-rest-destructuring` | manual | a query result is not rest-destructured or spread |
| `tanstack-query/no-unstable-deps` | manual | a query hook's whole result is not a React hook dependency |
| `tanstack-query/no-void-query-fn` | manual | a `queryFn` returns its data |
| `tanstack-query/infinite-query-property-order` | auto | `queryFn` before `getPreviousPageParam` / `getNextPageParam` |
| `tanstack-query/mutation-property-order` | auto | `onMutate` before `onError` / `onSettled` |

## How the ports differ

Every rule matches its source's documented behaviour on the source's own
examples. Where codelaw differs, the rule's `## Differences from the source`
section says so. The larger ones:

| rule | difference |
|---|---|
| `tanstack-query/no-void-query-fn` | upstream asks the TypeScript type checker; codelaw reports a `queryFn` written in place whose body cannot return a value |
| `tanstack-query/no-rest-destructuring` | upstream also finds a query result by its type under typed linting; codelaw follows direct hook calls only |
| `tanstack-query/exhaustive-deps` | `allowlist.types` matches the type names in the value's own annotation; nothing is inferred |
| the two `*-property-order` rules | the reordering is offered only when every moved option is a function, a method or a name |

## Not ported, and why

| rule | why |
|---|---|
| `@tanstack/query/prefer-query-options` | not in `recommended` (it is `recommended-strict`'s only addition), which is what this pack ports |
