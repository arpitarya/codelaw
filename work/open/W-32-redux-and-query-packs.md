---
type: Handoff
name: W-32
description: "Arpit, 2026-10-24: a Redux Toolkit pack and a React Query pack, on npm only."
item: W-32
filed: 2026-10-24
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5): steps 2–4 landed in the tree, uncommitted"
ball: arpit
---

# W-32 — A Redux Toolkit pack and a React Query pack, on npm

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The decider picks the names and the Redux Toolkit source (below) | the two packs can be scoped | ✅ 2026-10-24: `@codelaw/rules-redux-toolkit` (namespace `redux-toolkit`), `@codelaw/rules-tanstack-query` (namespace `tanstack-query`, renamed 2026-10-25 from `react-query`); source (a) |
| 2 | Port TanStack's `@tanstack/eslint-plugin-query` `recommended` into `packs/tanstack-query/` | TanStack Query rules | ✅ 2026-10-25: 7 rules, all of `recommended`, `native:` on the JS AST (`tanstack_query.rs`) |
| 3 | Write the Redux rules from the chosen source into `packs/redux-toolkit/` | Redux Toolkit rules | ✅ 2026-10-25: 12 rules, 9 `native:` (`redux_toolkit.rs`), 3 matcher-only |
| 4 | Add both to `FIRST_PARTY_PACKS` as `npm_only`, their namespaces to the reserved list and `namespace_pack`, `init` detection by `package.json` dependency | `init` installs them in a repo that uses them | ✅ 2026-10-25: `init` keys on `@tanstack/react-query`/`react-query` and `@reduxjs/toolkit`/`redux` |
| 5 | First hand publish of the two names, then `npm-trust.sh` (as W-31 step 7) | released | ✅ 2026-10-25: Arpit published both 0.2.0 tarballs from CI run 40189751330's `dist` (npm: "+ @codelaw/rules-tanstack-query@0.2.0", "+ @codelaw/rules-redux-toolkit@0.2.0") and bound both to `publish.yml` (`npm trust list`); live at 17:45 |

**Arpit, 2026-10-24:** *"i want a redux tool kit and react query pack as well in npm"*

**Model:** Opus: two rule ports with native matchers, the same work W-03 did for `react/*`.

## Goal

`@codelaw/rules-redux-toolkit` and `@codelaw/rules-tanstack-query` on npm, each with a `recommended` preset, ported or written under SR-RULE-PACKS decision 7 and L4.

## The decider decides

1. **Names.** Proposed: `@codelaw/rules-redux` (namespace `redux`) and `@codelaw/rules-query` (namespace `query`, since React Query is now TanStack Query). Or `rules-redux-toolkit` / `rules-react-query`.
2. **The Redux Toolkit source.** There is no official Redux Toolkit ESLint plugin. Options:
   - **(a)** codelaw's own rules from the official Redux Style Guide's Priority A and B rules (do not mutate state, reducers have no side effects, no non-serializable values in state or actions, one store per app, use Redux Toolkit, use the Immer-backed `createSlice`) — written fresh, `ported_from:` empty, each rule's README row naming its guide section;
   - **(b)** a port of the third-party `eslint-plugin-react-redux` (MIT), which covers react-redux usage more than Redux Toolkit;
   - **(c)** both: (a) as `redux/recommended`, (b) as `redux/react-redux`.

   Recommended: **(a)**, since it is the official guidance and is about Redux Toolkit itself.

**Arpit, 2026-10-24 (docket):** *"redux-toolkit + react-query, (a)"* — packs `@codelaw/rules-redux-toolkit` (namespace `redux-toolkit`) and `@codelaw/rules-react-query` (namespace `react-query`, in `packs/react-query/`); the Redux rules written fresh from the Redux Style Guide (a). Steps 2–4 use these names in place of `packs/query/` and `packs/redux/`.

**Arpit, 2026-10-25:** *"rename react-query > tanstack query"* — the query pack is `@codelaw/rules-tanstack-query`, namespace `tanstack-query`, in `packs/tanstack-query/`; its rules are `tanstack-query/*`. Supersedes the `react-query` name above.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · ruled | names and the Redux source ruled | ✅ 2026-10-24/25 |
| 2 · packs | `codelaw test packs/tanstack-query` and `packs/redux-toolkit` green; every rule has a fix class (L4) | ✅ 2026-10-25: 7 rules · 160 checks and 12 rules · 192 checks, 0 failed; `cargo test --workspace` 463 passed; clippy clean; `docs-sync.py --check` 11 packs, 438 rules |
| 3 · wired | `FIRST_PARTY_PACKS`, `RESERVED_NAMESPACES`, `init` and the pack tables in L6, SR-RULE-PACKS, SR-PACKAGES name them | ✅ 2026-10-25 (uncommitted; `package.py names` lists 19 npm names) |
| 4 · released | both on npm at the release version, trusted publishing bound | ✅ 2026-10-25 17:45: the registry serves `latest: 0.2.0` for both; `npm trust list` binds both to `publish.yml` — every DoD met, to close |

## Blockers

None: step 1 ruled 2026-10-24. Step 4 also amends [SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md)'s pack counts (nine packs → eleven, eight on npm → ten) under this ruling.

## Record affected

L6 decision 11 (npm's list grows to ten); SR-RULE-PACKS §1, decisions 2, 4 and 6; SR-PACKAGES §1 and decision 9; SR-RULE-FORMAT decision 1 (two reserved namespaces).

## Hazards

- **TanStack Query's rules read option objects** (`exhaustive-deps` on `queryKey`, `no-rest-destructuring`, `stable-query-client`): `native:` rules on the semantic model, like `react/exhaustive-deps`.
- **A Style Guide rule is guidance, not a check:** only the rules with a deterministic match ship (L5); the rest are listed as not ported, with why.

## Reference

- [TanStack ESLint Plugin Query](https://tanstack.com/query/latest/docs/eslint/eslint-plugin-query) · [eslint-plugin-react-redux](https://www.npmjs.com/package/eslint-plugin-react-redux) · the Redux Style Guide (redux.js.org/style-guide).
- [W-31](W-31-registry-pack-split.md) · [`work/OPEN-WORK.md`](../OPEN-WORK.md).
