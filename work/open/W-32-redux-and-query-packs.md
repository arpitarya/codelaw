---
type: Handoff
name: W-32
description: "Arpit, 2026-10-24: a Redux Toolkit pack and a React Query pack, on npm only."
item: W-32
filed: 2026-10-24
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-24
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-32 — A Redux Toolkit pack and a React Query pack, on npm

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The decider picks the names and the Redux Toolkit source (below) | the two packs can be scoped | 🔴 decider decides |
| 2 | Port TanStack's `@tanstack/eslint-plugin-query` `recommended` into `packs/query/` | React Query rules | ⏳ |
| 3 | Write the Redux rules from the chosen source into `packs/redux/` | Redux Toolkit rules | ⏳ |
| 4 | Add both to `FIRST_PARTY_PACKS` as `npm_only`, their namespaces to the reserved list and `namespace_pack`, `init` detection by `package.json` dependency | `init` installs them in a repo that uses them | ⏳ |
| 5 | First hand publish of the two names, then `npm-trust.sh` (as W-31 step 7) | released | ⏳ Arpit |

**Arpit, 2026-10-24:** *"i want a redux tool kit and react query pack as well in npm"*

**Model:** Opus: two rule ports with native matchers, the same work W-03 did for `react/*`.

## Goal

`@codelaw/rules-redux` and `@codelaw/rules-query` on npm, each with a `recommended` preset, ported or written under SR-RULE-PACKS decision 7 and L4.

## The decider decides

1. **Names.** Proposed: `@codelaw/rules-redux` (namespace `redux`) and `@codelaw/rules-query` (namespace `query`, since React Query is now TanStack Query). Or `rules-redux-toolkit` / `rules-react-query`.
2. **The Redux Toolkit source.** There is no official Redux Toolkit ESLint plugin. Options:
   - **(a)** codelaw's own rules from the official Redux Style Guide's Priority A and B rules (do not mutate state, reducers have no side effects, no non-serializable values in state or actions, one store per app, use Redux Toolkit, use the Immer-backed `createSlice`) — written fresh, `ported_from:` empty, each rule's README row naming its guide section;
   - **(b)** a port of the third-party `eslint-plugin-react-redux` (MIT), which covers react-redux usage more than Redux Toolkit;
   - **(c)** both: (a) as `redux/recommended`, (b) as `redux/react-redux`.

   Recommended: **(a)**, since it is the official guidance and is about Redux Toolkit itself.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · ruled | names and the Redux source ruled | 🔴 |
| 2 · packs | `codelaw test packs/query` and `packs/redux` green; every rule has a fix class (L4) | ⏳ |
| 3 · wired | `FIRST_PARTY_PACKS`, `RESERVED_NAMESPACES`, `init` and the pack tables in L6, SR-RULE-PACKS, SR-PACKAGES name them | ⏳ |
| 4 · released | both on npm at the release version, trusted publishing bound | ⏳ |

## Blockers

Step 1: the decider's ruling (inbox). W-31 lands first; this adds two rows to its table.

## Record affected

L6 decision 11 (npm's list grows to ten); SR-RULE-PACKS §1, decisions 2, 4 and 6; SR-PACKAGES §1 and decision 9; SR-RULE-FORMAT decision 1 (two reserved namespaces).

## Hazards

- **TanStack Query's rules read option objects** (`exhaustive-deps` on `queryKey`, `no-rest-destructuring`, `stable-query-client`): `native:` rules on the semantic model, like `react/exhaustive-deps`.
- **A Style Guide rule is guidance, not a check:** only the rules with a deterministic match ship (L5); the rest are listed as not ported, with why.

## Reference

- [TanStack ESLint Plugin Query](https://tanstack.com/query/latest/docs/eslint/eslint-plugin-query) · [eslint-plugin-react-redux](https://www.npmjs.com/package/eslint-plugin-react-redux) · the Redux Style Guide (redux.js.org/style-guide).
- [W-31](W-31-registry-pack-split.md) · [`work/OPEN-WORK.md`](../OPEN-WORK.md).
