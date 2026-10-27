---
type: Handoff
name: W-104
description: "Records first for type-aware rules (W-101, ruled 2026-10-27): amend L3, L5, L6 and L8; write SR-TYPES; the type: clause in SR-RULE-FORMAT; resolve SR-ARCHITECTURE's open line."
item: W-104
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-104 — Records for type-aware rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | **L3**: type-aware TS checks join the core when the project has TypeScript ≥ 7.1; rewrite the out-of-scope line and its consequence | the law allows it | ⏳ |
| 2 | **L5**: the project's TypeScript version is an input — same files, same TS, same codelaw → byte-identical | determinism stated | ⏳ |
| 3 | **L6**: Node at runtime for type rules only; the helper ships inside both engine packages | packaging stated | ⏳ |
| 4 | **L8**: hooks skip cold type rules with a note; `check_types_cold` / `check_types_warm` added as `unset` | budgets hold | ⏳ |
| 5 | **SR-TYPES** (new): helper, protocol, type-question vocabulary, TS lookup, ≥ 7.1 gate, warm model, tested range, `ts/type-error`, dedupe; SR-ARCHITECTURE's "not decided yet" line points at it; the `type:` clause in SR-RULE-FORMAT | the design of record | ⏳ |
| 6 | Ask the decider to confirm **L11**: the project's TypeScript is looked up like an adapter's binary, so no dependency | no hidden dependency | ⏳ |

**Arpit, 2026-10-27 (voice), W-101:** rulings R1–R8 in [the proposal](../proposals/type-aware-rules.md) §3 — TypeScript 7.1's own API through the project's installed copy, Node at runtime, rules in the `ts` pack, preview on the 7.1 RC, *"Go with the recommendation"* on version checks.

**Model:** Opus — amends four laws.

## Definition of done

- The four laws amended with the ruling quoted; SR-TYPES accepted (built: no); the register and ownership updated; L11's question answered by the decider.

## Record affected

SR-LAW-3, SR-LAW-5, SR-LAW-6, SR-LAW-8, SR-ARCHITECTURE, SR-RULE-FORMAT; new SR-TYPES.

## Hazards

- Law records outrank every other record: amend the laws first, then SR-TYPES quotes them, never the reverse.
