---
type: Handoff
name: W-43
description: "Arpit, 2026-10-25: more React hooks rules beyond rules-of-hooks and exhaustive-deps, from eslint-plugin-react-hooks 7 and @eslint-react's hooks rules."
item: W-43
filed: 2026-10-25
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-43 — More React hooks rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read the sources below; keep each candidate a matcher can decide without types or the React Compiler's analysis; record why any is dropped | a list grounded in upstream | ⏳ |
| 2 | Port each kept rule into `packs/react/` (`native:`, cases, an L4 fix class) | the rules | ⏳ |
| 3 | Run the pinned corpus (`corpus.py run --pack react`); a rule with a false positive there stays out of `react/recommended` | `recommended` stays quiet on real code | ⏳ |
| 4 | Pack README, CHANGELOG *Unreleased*, `docs-sync.py` | the docs count the new rules | ⏳ |

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

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · rules | every kept rule has cases and a fix class; `codelaw test packs/react` 0 failed | ⏳ |
| 2 · corpus | `corpus.py run --pack react` 0 failed; each new rule's count recorded | ⏳ |
| 3 · docs | pack README, CHANGELOG, `docs-sync.py --check` | ⏳ |

## Blockers

None.

## Record affected

[SR-RULE-PACKS](../../records/0101_rule-packs.md) (the pack's rule list) · [SR-ARCHITECTURE](../../records/0108_architecture.md) 15b (`codelaw-native`).
