---
type: Handoff
name: W-139
description: "Arpit, 2026-10-30 (voice): build into codelaw's packs every rule W-138's research marks worth porting, once he has reviewed it."
item: W-139
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-139 — Build: port the rules W-138 picks

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | W-138's research and Arpit's ruling | a decided list | ✅ 2026-10-30 — build all: 1,690 + 16 taste + 39 framework = **1,745** |
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

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · rules | every rule on the ruled port list (filter the CSVs in `work/proposals/rule-parity/` on `class = port` and the ruled `tranche`), in its pack, with valid and invalid cases and its fix class | ⏳ |
| 1b · links | the 29 equivalents marked `*` in the CSVs' `codelaw` column get their `ported_from:` line; the 3 id collisions the proposal names are checked first | ⏳ |
| 2 · presets | tranche A in `recommended`; the rest `preview`, in `strict` or opt-in presets; `js/unicorn` and a taste preset opt-in; B-048's `js/strict`, `react/a11y`, `py/strict`, pydocstyle preset | ⏳ |
| 2b · packs | five new PyPI packs `django`, `fastapi`, `airflow`, `numpy`, `pandas` for Ruff's 39 framework rules (`ruff.csv`, class `out-of-scope`); L6 amended; names W-142 | ⏳ |
| 3 · checks | `codelaw test` 0 failed on every pack; the layout check clean; corpus counts; L8 budgets measured, not loosened | ⏳ |
| 4 · docs | `scripts/rule-docs.py` pages, `docs-sync.py` counts, CHANGELOG, the proposal's before/after table | ⏳ |

## Blockers

None — W-138 ruled 2026-10-30. Rules W-144 adds join this build while it is open.

## Record affected

[SR-RULE-PACKS](../../records/0101_rule-packs.md) (the catalogue) · SR-FORMATTER if a `layout` rule exposes a formatter gap.

## Hazards

- Rewrite, never copy upstream text or cases (L2, L11); no new dependency.
- A rule that needs types waits for W-105's helper — do not fake it syntactically.
- Rule ids must not collide with names W-128–W-132 reserve.

## Reference

[W-138](../../archive/open/W-138-rule-parity-research.md) · W-03 · W-105
