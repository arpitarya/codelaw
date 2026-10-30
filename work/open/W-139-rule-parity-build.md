---
type: Handoff
name: W-139
description: "Arpit, 2026-10-30 (voice): build into codelaw's packs every rule W-138's and W-144's research marks worth porting — 2,572 rules, two new npm packs (testing, nextjs) and five PyPI framework packs."
item: W-139
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-139 — Build: port the rules W-138 and W-144 pick

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | W-138's research and Arpit's ruling | a decided list | ✅ 2026-10-30 — build all: 1,690 + 16 taste + 39 framework = **1,745**; round 2 (W-144) ruled the same day — **2,572 in all** |
| 2 | Write each rule fresh in its pack, with cases, a fix (auto, suggest or manual + skill) and its doc | the depth the single-language tools have | ⏳ |
| 3 | `codelaw test` on every pack, the corpus counts, the L8 budgets | nothing slower, nothing wrong | ⏳ |
| 4 | Counts per pack before and after, in the proposal and the README | the gap closed, measured | ⏳ |

**Arpit, 2026-10-30 (voice):** *"One big item for Opus to research it and once the research is done, then one big work item to build it."*

**Model:** Opus (Claude Code): hundreds of rules across every pack; split into parallel worktrees per pack only within L9's disk budget.

**W-138 ruled 2026-10-30 (Arpit, voice; written on *"file it."*):**

- **R1 — build everything, in one go** (*"I would suggest building out all the rules in single shot."*): all five tranches, 1,690 rules; plus ESLint's 16 `skip` taste rules (*"Let's build those 16."*), in an opt-in preset; plus Ruff's 39 framework rules (*"Let's build the 39 ones."*) as **five new packs** (*"Give each framework its own pack."*): `django`, `fastapi`, `airflow`, `numpy`, `pandas`, PyPI only as L6 ships `python` — names W-142. **1,745 rules.**
- **R2 — as recommended** (*"Go with your recommendation."*): tranche A joins each pack's `recommended` preset (on by default); everything else lands `preview`, in `strict` or opt-in presets; unicorn as its own opt-in preset; the 16 taste rules as their own opt-in preset.
- **R3 — as recommended:** `[format]` keys for `arrowParens`, `bracketSpacing`, `jsxSingleQuote`, `quoteProps`; Prettier's default kept for the other seven — W-141.
- **R4 — W-141 as filed.**
- **Also ruled the same call:** Biome's 89 Vue/Svelte/Astro/GraphQL rules → four new languages, W-143 (after W-139); more linters (sonarjs, test plugins, promise, jsdoc, regexp, full Pylint, Next.js, Tailwind) and TSLint's 30 without a mapped successor → W-144; a Tailwind pack with class sorting in the formatter → W-145.

**W-144 ruled 2026-10-30 (Arpit, voice; written on *"file it."*):**

- **R5 — a `testing` pack** (*"Let's create it."*): npm only, `@codelaw/rules-testing`, 182 rules (Jest, Vitest, Testing Library, Playwright); presets `testing/jest`, `testing/vitest`, `testing/testing-library`, `testing/playwright` — a project switches on the ones it uses. Python's test rules stay in `python`.
- **R6 — a `nextjs` pack** (*"The next JS pack. Let's create it."*): npm only, `@codelaw/rules-nextjs`, 22 rules; `nextjs/recommended` on when installed.
- **R7 — defaults:** SonarJS and Pylint off (*"Off by default. For Sonar.js or Pylint."*): opt-in presets `js/sonarjs` (201) and `py/pylint` (298) inside their packs, not new packs (*"Perfect."*); jsdoc off, `js/jsdoc` (71), as recommended; regexp and promise on (*"Yes."*) — their 70 upstream-recommended rules join `js/recommended`, the other 27 `preview`. Each opt-in preset mirrors its source's recommended set; this overrides R2 for round 2's tranche A.
- **R8 — TSLint's 6** (*"Let's file all this."*): `js/file-header`, `js/no-default-import`, `js/no-unnecessary-callback-wrapper` (suggest) in `js`; `ts/no-mergeable-namespace` in `ts`; `js/comment-type`, `js/switch-final-break` in the taste preset.
- **The count:** every `port` row in the CSVs (2,574) less the `tailwind` pack's 8 (W-145 builds those with the pack), plus TSLint's 6 = **2,572**. Codelaw goes from 689 to about 3,260.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · rules | every rule on the ruled port list (the CSVs in `work/proposals/rule-parity/` on `class = port`, `proposed_pack` not `tailwind`; plus `tslint-recheck.csv`'s 6), in its pack, with valid and invalid cases and its fix class | ⏳ |
| 1b · links | the 29 equivalents marked `*` in the CSVs' `codelaw` column get their `ported_from:` line; the 3 id collisions the proposal names are checked first | ⏳ |
| 2 · presets | tranche A in `recommended`; the rest `preview`, in `strict` or opt-in presets; `js/unicorn` and a taste preset opt-in; B-048's `js/strict`, `react/a11y`, `py/strict`, pydocstyle preset; round 2: `js/sonarjs`, `js/jsdoc`, `py/pylint` opt-in, regexp's and promise's tranche A in `js/recommended`, a `testing/*` preset per framework | ⏳ |
| 2b · packs | five new PyPI packs `django`, `fastapi`, `airflow`, `numpy`, `pandas` for Ruff's 39 framework rules (`ruff.csv`, class `out-of-scope`); L6 amended; names W-142 | ⏳ |
| 2c · npm packs | `testing` and `nextjs`, npm only; L6 amended (decision 11's table, the counts, CLAUDE.md's L6 line); names W-147 | ⏳ |
| 3 · checks | `codelaw test` 0 failed on every pack; the layout check clean; corpus counts; L8 budgets measured, not loosened | ⏳ |
| 4 · docs | `scripts/rule-docs.py` pages, `docs-sync.py` counts, CHANGELOG, the proposal's before/after table | ⏳ |

## Blockers

None — W-138 and W-144 ruled 2026-10-30.

## Record affected

[SR-RULE-PACKS](../../records/0101_rule-packs.md) (the catalogue) · SR-FORMATTER if a `layout` rule exposes a formatter gap.

## Hazards

- Rewrite, never copy upstream text or cases (L2, L11); no new dependency. SonarJS (LGPL-3.0) and Pylint (GPL-2.0): behaviour only — never their messages, tests or examples.
- A rule that needs types waits for W-105's helper — do not fake it syntactically.
- Rule ids must not collide with names W-128–W-132 reserve.

## Reference

[W-138](../../archive/open/W-138-rule-parity-research.md) · [W-144](../../archive/open/W-144-rule-parity-round-2.md) · [W-145](W-145-tailwind-pack-and-class-sorting.md) · [W-142](W-142-framework-pack-names.md) · [W-147](W-147-testing-nextjs-npm-first-publish.md) · W-03 · W-105
