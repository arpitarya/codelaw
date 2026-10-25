---
type: Handoff
name: W-43
description: "Arpit, 2026-10-25: more React hooks rules beyond rules-of-hooks and exhaustive-deps, from eslint-plugin-react-hooks 7 and @eslint-react's hooks rules."
item: W-43
filed: 2026-10-25
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5), 2026-10-25: closed — merged and verified on main"
ball: done
---

# W-43 — More React hooks rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read the sources below; keep each candidate a matcher can decide without types or the React Compiler's analysis; record why any is dropped | a list grounded in upstream | ✅ 2026-10-25 — eslint-plugin-react-hooks 7.1.1 and @eslint-react/eslint-plugin 5.28.8 (eslint-plugin-react-x 5.28.8) read from their npm tarballs; 10 kept, the rest dropped with reasons — §Step 1 below |
| 2 | Port each kept rule into `packs/react/` (`native:`, cases, an L4 fix class) | the rules | ✅ 2026-10-25 — `crates/codelaw-native/src/react_hooks.rs`; a page per rule with four-language examples and *Differences from the source*; 9 `manual`, 1 `suggest` |
| 3 | Run the pinned corpus (`corpus.py run --pack react`); a rule with a false positive there stays out of `react/recommended` | `recommended` stays quiet on real code | ✅ 2026-10-25 — every finding read: one false positive (`no-unnecessary-use-prefix` missed a hook called as an arrow's expression body, zustand ×2), fixed and unit-tested; after it, none — all ten in `react/recommended` |
| 4 | Pack README, CHANGELOG *Unreleased*, `docs-sync.py` | the docs count the new rules | ✅ 2026-10-25 — the preset row (29), *How the ports differ*, *Not ported, and why*, NOTICE; README regenerated (451 rules) |

**Arpit, 2026-10-25:** *"I want more React query hooks and React hooks. Rules."*

**Model:** Opus — porting compiler-derived rules to syntax needs judgment on what is decidable.

## Sources

- `eslint-plugin-react-hooks` 7.x — the rules beside `rules-of-hooks` and `exhaustive-deps` (`set-state-in-render`, `set-state-in-effect`, `refs`, `purity`, `static-components`, `use-memo`, `globals`, `immutability`, …).
- `@eslint-react/eslint-plugin` — its hooks rules (`no-direct-set-state-in-use-effect`, `no-unnecessary-use-callback`, `no-unnecessary-use-memo`, `prefer-use-state-lazy-initialization`, `no-unnecessary-use-prefix`).

## Candidates (step 1 keeps or drops each)

| rule | what it flags |
|---|---|
| `react/no-set-state-in-render` | a state setter called unconditionally in a component's body |
| `react/no-set-state-in-effect` | a setter called synchronously at the top of an effect |
| `react/no-ref-in-render` | `ref.current` read or written during render |
| `react/purity` | `Math.random()`, `Date.now()`, `new Date()` during render |
| `react/no-nested-components` | a component defined inside another component's body |
| `react/use-memo-returns` | a `useMemo` callback that returns nothing, or is `async` |
| `react/prefer-lazy-state-init` | `useState(expensiveCall())` where `useState(() => …)` would do |
| `react/no-unnecessary-use-prefix` | a `use…` function that calls no hook |

W-41's `react/no-context` is the same pack; it lands separately (its own item).

## Step 1 — what upstream has, and what was kept

Read 2026-10-25 from the npm tarballs of `eslint-plugin-react-hooks@7.1.1` (the compiler's
`getRuleForCategoryImpl`, its validation passes and its impure-function signatures) and
`@eslint-react/eslint-plugin@5.28.8`, whose React rules live in `eslint-plugin-react-x@5.28.8`.
The candidates' old @eslint-react names are gone in 5.x: `no-direct-set-state-in-use-effect` is
`set-state-in-effect`, `prefer-use-state-lazy-initialization` is part of `use-state`, and
`no-unnecessary-use-callback` / `no-unnecessary-use-memo` were removed upstream.

| candidate → codelaw rule | upstream | kept? | fix |
|---|---|---|---|
| `react/no-set-state-in-render` | react-hooks `set-state-in-render`; @eslint-react `set-state-in-render` | ✅ the setter is known by its `useState` declaration in the same function | manual |
| `react/no-set-state-in-effect` | react-hooks `set-state-in-effect`; @eslint-react `set-state-in-effect` | ✅ a direct setter call in the effect callback's own body | manual |
| `react/no-ref-in-render` | react-hooks `refs`; @eslint-react `refs` | ✅ refs from `useRef` in the same function; the documented lazy-init `if` allowed | manual |
| `react/purity` | react-hooks `purity` (`Math.random`, `Date.now`, `performance.now`); @eslint-react `purity` | ✅ those three, `crypto.randomUUID`/`getRandomValues` and `new Date()`; the rest of @eslint-react's list (DOM, storage, timers, network) left out — side effects, not impurity | manual |
| `react/no-nested-components` | react-hooks `static-components`; @eslint-react `no-nested-component-definitions` | ✅ a PascalCase function rendering JSX inside a component or `render()` | manual |
| `react/use-memo` (was `use-memo-returns`) | react-hooks `use-memo` + `void-use-memo`; @eslint-react `use-memo` | ✅ async/generator, parameters, no returned value, unused result — renamed to upstream's name because it checks more than the return | manual |
| `react/prefer-lazy-state-init` | @eslint-react `use-state` (its lazy-initialisation check) | ✅ | **suggest**: `useState(x())` → `useState(() => x())` changes behaviour when the value is itself a function |
| `react/no-unnecessary-use-prefix` | @eslint-react `no-unnecessary-use-prefix` | ✅ | manual (callers rename) |
| `react/globals` | react-hooks `globals`; @eslint-react `globals` | ✅ assignments and updates to an unbound or module-level name, in render | manual |
| `react/error-boundaries` | react-hooks `error-boundaries`; @eslint-react `error-boundaries` | ✅ `try` around a returned JSX value or `use(…)` | manual |
| — | react-hooks `immutability`; @eslint-react `immutability` | ❌ mutation through methods (`push`, `sort`) and aliases needs types and alias analysis | — |
| — | react-hooks `preserve-manual-memoization` | ❌ asks whether the compiler's own memoisation matches the code's | — |
| — | react-hooks `config`, `gating` | ❌ validate the compiler's configuration, not code | — |
| — | react-hooks `incompatible-library` | ❌ a list of library APIs the compiler can't memoise; matters only with the compiler | — |
| — | react-hooks `unsupported-syntax`; @eslint-react `unsupported-syntax` | ❌ syntax the compiler can't compile (`eval`, `with`); `js/*` covers the language-level cases | — |
| — | react-hooks rules off in every preset: `capitalized-calls`, `memoized-effect-dependencies`, `exhaustive-effect-dependencies`, `no-deriving-state-in-effects`, `fbt`, `hooks`, `invariant`, `rule-suppression`, `syntax`, `todo`, `memo-dependencies` | ❌ experimental or internal; `hooks` and the dependency rules duplicate `rules-of-hooks` and `exhaustive-deps` | — |
| — | @eslint-react `use-state`'s destructuring and setter-name checks | ❌ naming conventions, not defects; a `react/conventions` candidate | — |
| — | @eslint-react's non-hooks rules (`no-nested-lazy-component-declarations`, `no-use-context`, `no-forward-ref`, …) | ❌ outside this item, which is about hooks; `react/no-context` is W-41 | — |

## Step 3 — the corpus (zustand, jotai, full-stack-fastapi-template)

Every finding was read. Before the fix, `no-unnecessary-use-prefix` reported zustand's
`const useBoundStore = (selector) => useStore(api, selector)` twice — the arrow's expression body
is itself the hook call and was not searched. Fixed (the body is searched too) with a unit case.

| rule | count | what they are |
|---|---|---|
| `react/no-set-state-in-effect` | 5 | `setResolvedTheme`, `setIsMobile`, `setHasMounted(true)` on mount, two in tests — the patterns upstream reports |
| `react/no-ref-in-render` | 4 | jotai's `Provider` reads `storeRef.current` after its lazy init, and three test hooks; two carry `eslint-disable-next-line react-hooks/refs` upstream |
| `react/no-nested-components` | 2 | `Input` and `Uppercase` defined inside jotai's website `CoreDemo` |
| `react/purity` | 2 | `new Date().getFullYear()` in two footers |
| `react/no-unnecessary-use-prefix` | 1 | `useCustomToast` calls only `toast.*` |
| the other five | 0 | the corpus has none of these patterns |

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · rules | every kept rule has cases and a fix class; `codelaw test packs/react` 0 failed | ✅ `codelaw test packs/react`: 32 rules · 469 checks · 0 failed; `cargo test -p codelaw-native` 127 passed (10 new `react_hooks::tests`) |
| 2 · corpus | `corpus.py run --pack react` 0 failed; each new rule's count recorded | ✅ 646 files · 108 findings · 32 rules · 535 checks · 0 failed; counts in `packs/react/corpus.toml` and §Step 3 |
| 3 · docs | pack README, CHANGELOG, `docs-sync.py --check` | ✅ `docs-sync: 11 packs, 451 rules, 21 commands` |

## Blockers

None.

## Record affected

[SR-RULE-PACKS](../../records/0101_rule-packs.md) (the pack's rule list) · [SR-ARCHITECTURE](../../records/0108_architecture.md) 15b (`codelaw-native`).

**As landed (2026-10-25):** SR-RULE-PACKS decision 7 gains a sub-bullet under *Not ported* (a compiler rule whose core case one file shows is ported for that case; the 8 of 14 kept and why the 6 were not); SR-ARCHITECTURE 15b gains one for `react_hooks.rs`.
