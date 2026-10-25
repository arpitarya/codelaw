---
type: Handoff
name: W-42
description: "Arpit, 2026-10-25: more TanStack Query hook rules beyond @tanstack/eslint-plugin-query's seven, from TanStack's own v5 docs."
item: W-42
filed: 2026-10-25
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5), 2026-10-25: closed — merged to main as b4a6ac4"
ball: done
---

# W-42 — More TanStack Query hook rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read the sources below; keep each candidate that TanStack's docs state as a rule and that a matcher can decide without types; record why any is dropped | a list grounded in the docs, not invented | ✅ 2026-10-25 — five candidates kept (one split in two), one dropped; §Step 1 |
| 2 | Port each kept rule into `packs/tanstack-query/` (`native:` on the JS AST, cases, an L4 fix class; `auto` only for a pure rename) | the rules | ✅ six rules in `tanstack_query.rs`, examples in four languages, unit tests |
| 3 | Run the pinned corpus (`corpus.py run --pack tanstack-query`); a rule with a false positive there stays out of `recommended` | `recommended` stays quiet on real code | ✅ the pack had no corpus: three repos added; one false positive found and fixed, one option dropped; §Corpus |
| 4 | Pack README, CHANGELOG *Unreleased*, `docs-sync.py` | the docs count the new rules | ✅ |

**Arpit, 2026-10-25:** *"I want more React query hooks and React hooks. Rules. When I say React query, I was talking about TanStack query."* — the pack already holds all seven rules of `@tanstack/eslint-plugin-query`, so the new ones come from TanStack's v5 documentation.

**Model:** Sonnet or Opus — rule design from docs, matchers like `tanstack_query.rs`.

## Sources

- TanStack Query v5 — *Migrating to TanStack Query v5* (removed and renamed options, the object-only signature).
- TanStack Query v5 guides — *Query Keys*, *Invalidations from Mutations*, *Updates from Mutation Responses*, *Suspense*.

## Candidates (step 1 keeps or drops each, with the doc's words)

| rule | what it flags | fix |
|---|---|---|
| `tanstack-query/no-removed-v4-options` | `onSuccess` / `onError` / `onSettled` on `useQuery`; `cacheTime`, `useErrorBoundary`, `keepPreviousData: true` | `auto` for the renames (`gcTime`, `throwOnError`), `manual` otherwise |
| `tanstack-query/object-syntax` | `useQuery(key, fn, opts)` and the other positional calls v5 removed | `auto` where the arguments are plain |
| `tanstack-query/array-query-key` | a `queryKey` that is not an array | `suggest` |
| `tanstack-query/return-invalidate-in-mutation` | `invalidateQueries` in a mutation's `onSuccess`/`onSettled` neither returned nor awaited | `suggest` |
| `tanstack-query/no-mutating-set-query-data` | a `setQueryData` updater that mutates its argument (`push`, assignment) | `manual` |
| `tanstack-query/no-suspense-only-options` | `enabled`, `placeholderData` on `useSuspenseQuery` / `useSuspenseInfiniteQuery` | `suggest` |

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · rules | every kept rule has cases and a fix class; `codelaw test packs/tanstack-query` 0 failed | ✅ 13 rules, 304 checks, 0 failed |
| 2 · corpus | `corpus.py run --pack tanstack-query` 0 failed; each new rule's count recorded | ✅ 0 failed; counts in §Corpus and `packs/tanstack-query/corpus.toml` |
| 3 · docs | pack README, CHANGELOG, `docs-sync.py --check` | ✅ 11 packs, 446 rules |

## Step 1 — kept and dropped, in the docs' words

Read from TanStack/query `main` (`docs/framework/react/guides/`), 2026-10-25.

| candidate | verdict | the doc's words | rule |
|---|---|---|---|
| `no-removed-v4-options` (renames) | **kept, split out** as `no-renamed-v4-options`, `auto` (a pure key rename) | *Migrating to v5:* "Rename `cacheTime` to `gcTime`"; "The `useErrorBoundary` option has been renamed to `throwOnError`" | `tanstack-query/no-renamed-v4-options` |
| `no-removed-v4-options` (removals) | **kept**, `manual` (each needs other code) | "`onSuccess`, `onError` and `onSettled` have been removed from Queries. They haven't been touched for Mutations"; "Removed `keepPreviousData` in favor of `placeholderData` identity function"; "The `isDataEqual` option has been removed"; "Removed custom `context` prop" | `tanstack-query/no-removed-v4-options` |
| — `suspense: true` in it | **dropped from the rule** | the guide says "The experimental `suspense: boolean` flag on the query hooks has been removed", but the corpus showed `@tanstack/query-core` v5 still declares `suspense?: boolean` on observer options and `useBaseQuery` still reads it, so it is not ignored at runtime | — |
| `object-syntax` | **kept**, `suggest` (the call's shape changes, so not a rename) | "Supports a single signature, one object … now we only support the object format" | `tanstack-query/object-syntax` |
| `array-query-key` | **kept**, `suggest` | *Query Keys:* "Query keys have to be an Array at the top level" | `tanstack-query/array-query-key` |
| `return-invalidate-in-mutation` | **dropped** | *Invalidations from Mutations:* "Returning a Promise on `onSuccess` makes sure the data is updated before the mutation is entirely complete" and "You can wire up your invalidations to happen using any of the callbacks" — it describes what returning does, a choice, not a rule | — (in the pack README's "not ported") |
| `no-mutating-set-query-data` | **kept**, `manual` | *Updates from Mutation Responses:* "Updates via `setQueryData` must be performed in an immutable way. **DO NOT** attempt to write directly to the cache by mutating data … in place" | `tanstack-query/no-mutating-set-query-data` |
| `no-suspense-only-options` | **kept, renamed** `no-unsupported-suspense-options` (the options are the ones the suspense hooks lack), `suggest` (removing `enabled` makes the query always run) | *Suspense:* "you therefore can't conditionally enable / disable the Query"; "`placeholderData` also doesn't exist for this Query"; "you can't change `throwOnError`" | `tanstack-query/no-unsupported-suspense-options` |

## Corpus

`packs/tanstack-query` had no corpus (`corpus.py`'s `PACKS` left it out). Added: `full-stack-fastapi-template`
(already pinned), `bulletproof-react` @ `9506629e` and TanStack/query @ `cb3c3d5b` (all MIT). 4151 files.

| rule | count | where |
|---|---|---|
| `no-renamed-v4-options` | 3 | the `rename-properties` codemod's v4 input fixture — true positives |
| `no-removed-v4-options` | 76 | the `keep-previous-data` codemod's v4 fixtures, input and the output it leaves untransformed — true positives |
| `object-syntax` | 193 | the v4 and v5 codemods' v4 fixtures, including `setQueriesData([…], …)` the v5 codemod leaves alone — true positives |
| `array-query-key` | 7 | 6 in v4 codemod fixtures; 1 a `@ts-expect-error` type test passing `'not-an-array'` on purpose — true positives |
| `no-mutating-set-query-data` | 0 | — |
| `no-unsupported-suspense-options` | 0 | — |

The two real v5 apps produce no finding from the six. The first run had one false positive —
`useMutation(() => ({ … }))` in `examples/solid`: Solid's and Svelte's hooks take a function returning the
options — fixed (a callee imported from a `solid`/`svelte` package is not read as v4's `useMutation(fn)`),
and one `suspense: true` in a `query-core` test, which led to dropping `suspense` (above). With both
gone, all six are in `tanstack-query/recommended` (13 rules; `no-mutating-set-query-data` at `warn`, the
rest `error`).

**For the decider:** `codelaw init` adds this pack for `react-query` (v3) and any `@tanstack/react-query`,
v4 included; on v4 code `object-syntax` and the two `v4-options` rules report every v4 call, correctly for
v5 but loudly for a repo not yet migrated. Not a blocker; named so the preset's reach is a choice.

## Blockers

None.

## Record affected

[SR-RULE-PACKS](../../records/0101_rule-packs.md) (the pack's rule list) · [SR-ARCHITECTURE](../../records/0108_architecture.md) 15b (`codelaw-native`).
