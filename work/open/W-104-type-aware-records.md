---
type: Handoff
name: W-104
description: "Records first for type-aware rules (W-101, ruled 2026-10-27): amend L3, L5, L6 and L8; write SR-TYPES; the type: clause in SR-RULE-FORMAT; resolve SR-ARCHITECTURE's open line."
item: W-104
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5)"
ball: arpit
---

# W-104 — Records for type-aware rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | **L3**: type-aware TS checks join the core when the project has TypeScript ≥ 7.1; rewrite the out-of-scope line and its consequence | the law allows it — decision 9, the §1 law text, decision 6's asymmetry, the out-of-scope line now Python's only, Consequences | ✅ |
| 2 | **L5**: the project's TypeScript version is an input — same files, same TS, same codelaw → byte-identical | determinism stated — the law text, decision 2's third input, decision 3 (the helper fetches nothing) | ✅ |
| 3 | **L6**: Node at runtime for type rules only; the helper ships inside both engine packages | packaging stated — the law text and decision 3's one runtime exception; the runtime fence's no-Node note | ✅ |
| 4 | **L8**: hooks skip cold type rules with a note; `check_types_cold` / `check_types_warm` added as `unset` | budgets hold — decision 1's two rows, decision 8; they enter `perf/budgets.toml` with the bench that times them (W-105) | ✅ |
| 5 | **SR-TYPES** (new): helper, protocol, type-question vocabulary, TS lookup, ≥ 7.1 gate, warm model, tested range, `ts/type-error`, dedupe; SR-ARCHITECTURE's "not decided yet" line points at it; the `type:` clause in SR-RULE-FORMAT | the design of record — [SR-TYPES](../../records/0117_types.md) (0117, accepted, built: no; owns nothing until W-105 adds the helper); SR-RULE-FORMAT decision 3f; SR-ARCHITECTURE's Consequences line and decision 28; DESIGN §3.6 and the risks table | ✅ |
| 6 | Ask the decider to confirm **L11**: the project's TypeScript is looked up like an adapter's binary, so no dependency | no hidden dependency — asked in OPEN-WORK's inbox, with the second question it raised: codelaw's own CI and tests need a TypeScript 7.1 copy | 🔴 the decider |

**Arpit, 2026-10-27 (voice), W-101:** rulings R1–R8 in [the proposal](../proposals/type-aware-rules.md) §3 — TypeScript 7.1's own API through the project's installed copy, Node at runtime, rules in the `ts` pack, preview on the 7.1 RC, *"Go with the recommendation"* on version checks.

**Model:** Opus — amends four laws.

## Definition of done

- ✅ The four laws amended with the ruling quoted (each names W-101's ruling of 2026-10-27 and quotes its words from [the proposal](../proposals/type-aware-rules.md) §3, R1–R8; L8's change rests on R1–R8 and the proposal's §4–§5, which no single spoken sentence names).
- ✅ SR-TYPES accepted (built: no); the register row added. **The Ownership table has no SR-TYPES row:** it lists only components that exist (SR-WORK-OWNERSHIP decision 2), and SR-TYPES owns none until W-105 builds the helper.
- 🔴 L11's question answered by the decider — in the inbox.

## Left

1. **The decider answers the L11 inbox row** (two questions: the project's TypeScript is no dependency; codelaw's own CI/test copy of TypeScript 7.1 is one, and needs a row before W-105 lands it).
2. Then: write his words into SR-TYPES decision 5 (and, if he ratifies the CI copy, a D-row in L11 §3), and close.

## Record affected

SR-LAW-3, SR-LAW-5, SR-LAW-6, SR-LAW-8, SR-ARCHITECTURE, SR-RULE-FORMAT; new SR-TYPES.

## Hazards

- Law records outrank every other record: amend the laws first, then SR-TYPES quotes them, never the reverse.
