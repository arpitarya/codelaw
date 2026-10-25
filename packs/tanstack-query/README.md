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
| `tanstack-query/recommended` | 13 | @tanstack/eslint-plugin-query 5.104.5 `recommended` (all 7), at its severities, and the 6 rules from TanStack Query v5's guides below |

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

Six more rules come from TanStack Query v5's own guides — *Migrating to TanStack Query v5*, *Query Keys*,
*Updates from Mutation Responses* and *Suspense* — where a guide states a rule a matcher can decide
without types (W-42). Each rule page quotes the guide's words.

| rule | fix | checks |
|---|---|---|
| `tanstack-query/no-renamed-v4-options` | auto | `gcTime` and `throwOnError`, not v4's `cacheTime` and `useErrorBoundary` |
| `tanstack-query/no-removed-v4-options` | manual | no `onSuccess` / `onError` / `onSettled`, `keepPreviousData`, `isDataEqual` or `context` on a query |
| `tanstack-query/object-syntax` | suggest | one options object, not v4's positional `(key, fn, options)` |
| `tanstack-query/array-query-key` | suggest | a `queryKey` is an array |
| `tanstack-query/no-mutating-set-query-data` | manual | a `setQueryData` updater doesn't change the cached data in place |
| `tanstack-query/no-unsupported-suspense-options` | suggest | no `enabled`, `placeholderData` or `throwOnError` on a suspense query |

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
| a returned `invalidateQueries` in a mutation callback (W-42 candidate) | *Invalidations from Mutations* says what returning the promise does ("makes sure the data is updated before the mutation is entirely complete") and that "you can wire up your invalidations … using any of the callbacks"; it describes a choice, not a rule |
