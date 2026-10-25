---
type: Handoff
name: W-42
description: "Arpit, 2026-10-25: more TanStack Query hook rules beyond @tanstack/eslint-plugin-query's seven, from TanStack's own v5 docs."
item: W-42
filed: 2026-10-25
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-42 — More TanStack Query hook rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read the sources below; keep each candidate that TanStack's docs state as a rule and that a matcher can decide without types; record why any is dropped | a list grounded in the docs, not invented | ⏳ |
| 2 | Port each kept rule into `packs/tanstack-query/` (`native:` on the JS AST, cases, an L4 fix class; `auto` only for a pure rename) | the rules | ⏳ |
| 3 | Run the pinned corpus (`corpus.py run --pack tanstack-query`); a rule with a false positive there stays out of `recommended` | `recommended` stays quiet on real code | ⏳ |
| 4 | Pack README, CHANGELOG *Unreleased*, `docs-sync.py` | the docs count the new rules | ⏳ |

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
| 1 · rules | every kept rule has cases and a fix class; `codelaw test packs/tanstack-query` 0 failed | ⏳ |
| 2 · corpus | `corpus.py run --pack tanstack-query` 0 failed; each new rule's count recorded | ⏳ |
| 3 · docs | pack README, CHANGELOG, `docs-sync.py --check` | ⏳ |

## Blockers

None.

## Record affected

[SR-RULE-PACKS](../../records/0101_rule-packs.md) (the pack's rule list) · [SR-ARCHITECTURE](../../records/0108_architecture.md) 15b (`codelaw-native`).
