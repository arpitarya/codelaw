---
type: Handoff
name: W-144
description: "Arpit, 2026-10-30 (voice): count and match the rules of more famous linters — sonarjs, the test plugins, promise, jsdoc, regexp, full Pylint, Next.js, Tailwind — and recheck TSLint's 30 rules without a mapped successor; reimplemented fresh, never copied."
item: W-144
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: none
---

# W-144 — Research: rule parity, round 2

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read every rule of the tools below from their own packages, as W-138 did | full counts | ✅ 2026-10-30 — 1,211 rules from 12 packages |
| 2 | Match, class and propose pack/id/fix — `work/proposals/rule-parity/` method, extended | the port list | ✅ 879 to port; 146 counted elsewhere (Pylint→Ruff 80, Vitest→Jest 55, SonarJS 7, Tailwind 4) |
| 3 | Recheck TSLint's 30 rules with no mapped successor against today's rules | what is still worth building | ✅ 6 worth building (`tslint-recheck.csv`) |
| 4 | Add to the proposal; the ports join W-139 if it is open, else a follow-on build | one build, as ruled | ✅ ruled 2026-10-30 — round 2 joins W-139 (Tailwind's 8 in W-145); **closed** |

**Arpit, 2026-10-30 (voice):** on other famous linters — *"Let's re-implement it."* — and Next.js and Tailwind: *"Let's do both."*

**Researched 2026-10-30 (Cowork, Opus 5.5):** *"research w-144"* → [proposal §11](../../work/proposals/rule-parity.md). Both rounds: **2,574 to port** (+6 TSLint = 2,580); codelaw would go from 689 to about 3,270. 52 Biome rules move to their sources now read. The CSVs also carry the 10-30 rulings (taste rules and Ruff's framework rules as `port`).

**Model:** Cowork (Opus 5.5) — research (rule 37a).

**Ruled 2026-10-30 (Arpit, voice; written on *"file it."*):**

- **R5 — a `testing` pack** (*"Let's create it."*): npm only, `@codelaw/rules-testing`, 182 rules, a preset per framework.
- **R6 — a `nextjs` pack** (*"The next JS pack. Let's create it."*): npm only, `@codelaw/rules-nextjs`, 22 rules, on when installed.
- **R7 — defaults:** SonarJS and Pylint off (*"Off by default. For Sonar.js or Pylint."*), opt-in presets `js/sonarjs` and `py/pylint` inside their packs (*"Perfect."*); jsdoc off as recommended (`js/jsdoc`); regexp and promise on (*"Yes."*) — their 70 recommended rules join `js/recommended`.
- **R8 — TSLint's 6, built** in `js` and `ts`, two in the taste preset; no new package (*"Let's file all this."*).

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
| 1 · counts | one CSV per tool in `work/proposals/rule-parity/`, `summary.json` extended | ✅ |
| 2 · proposal | `rule-parity.md` §4–6 extended; new packs and their registries named | ✅ §11 |
| 3 · TSLint | the 30 classed against today's ESLint, typescript-eslint, Biome and codelaw rules | ✅ |
| 4 · ruling | R5 `testing` pack · R6 `nextjs` pack · R7 round 2 as opt-in presets · R8 TSLint's 6 | ✅ 2026-10-30 |

## Blockers

None.

## Record affected

No SR affected — research.

## Hazards

- **LGPL and GPL sources:** read what a rule does; never paste its text, messages, tests or examples (L2).
- New packs mean L6 amendments and registry names — name them in the proposal so the decider rules once.

## Reference

[W-138](W-138-rule-parity-research.md) · [proposal](../../work/proposals/rule-parity.md) · [W-139](../../work/open/W-139-rule-parity-build.md) · [W-145](../../work/open/W-145-tailwind-pack-and-class-sorting.md) · [W-147](../../work/open/W-147-testing-nextjs-npm-first-publish.md)
