---
type: Handoff
name: W-144
description: "Arpit, 2026-10-30 (voice): count and match the rules of more famous linters — sonarjs, the test plugins, promise, jsdoc, regexp, full Pylint, Next.js, Tailwind — and recheck TSLint's 30 rules without a mapped successor; reimplemented fresh, never copied."
item: W-144
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: cowork
---

# W-144 — Research: rule parity, round 2

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read every rule of the tools below from their own packages, as W-138 did | full counts | ⏳ |
| 2 | Match, class and propose pack/id/fix — `work/proposals/rule-parity/` method, extended | the port list | ⏳ |
| 3 | Recheck TSLint's 30 rules with no mapped successor against today's rules | what is still worth building | ⏳ |
| 4 | Add to the proposal; the ports join W-139 if it is open, else a follow-on build | one build, as ruled | ⏳ |

**Arpit, 2026-10-30 (voice):** on other famous linters — *"Let's re-implement it."* — and Next.js and Tailwind: *"Let's do both."*

**Model:** Cowork (Opus 5.5) — research (rule 37a).

## Goal

| tool | licence | note |
|---|---|---|
| eslint-plugin-sonarjs | LGPL-3.0 | behaviour only: write every rule, case and doc fresh |
| eslint-plugin-jest, @vitest/eslint-plugin, eslint-plugin-testing-library, eslint-plugin-playwright | MIT | likely a test pack — propose it |
| eslint-plugin-promise, eslint-plugin-jsdoc, eslint-plugin-regexp | ISC, BSD-3, MIT | into `js`/`ts` |
| Pylint (all of it) | GPL-2.0 | behaviour only; Ruff's `PL` already matched in W-138 |
| @next/eslint-plugin-next | MIT | a Next.js pack (npm) |
| eslint-plugin-tailwindcss / better-tailwindcss | MIT | the bug-catching rules only (conflicting, unknown classes) into the Tailwind pack (W-145); class order is the formatter's |
| TSLint's 30 without a mapped successor | Apache-2.0 | `tslint.csv`, note `no ESLint successor` |

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · counts | one CSV per tool in `work/proposals/rule-parity/`, `summary.json` extended | ⏳ |
| 2 · proposal | `rule-parity.md` §4–6 extended; new packs and their registries named | ⏳ |
| 3 · TSLint | the 30 classed against today's ESLint, typescript-eslint, Biome and codelaw rules | ⏳ |

## Blockers

None.

## Record affected

No SR affected — research.

## Hazards

- **LGPL and GPL sources:** read what a rule does; never paste its text, messages, tests or examples (L2).
- New packs mean L6 amendments and registry names — name them in the proposal so the decider rules once.

## Reference

[W-138](../../archive/open/W-138-rule-parity-research.md) · [proposal](../proposals/rule-parity.md) · [W-139](W-139-rule-parity-build.md) · [W-145](W-145-tailwind-pack-and-class-sorting.md)
