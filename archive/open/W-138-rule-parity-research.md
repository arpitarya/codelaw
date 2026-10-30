---
type: Handoff
name: W-138
description: "Arpit, 2026-10-30 (voice): research every rule of ESLint, typescript-eslint, TSLint, Ruff, Prettier, Biome, stylelint, html-validate and markdownlint against codelaw's packs, and say which to port."
item: W-138
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: none
---

# W-138 — Research: every rule of nine tools, matched against codelaw

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | List every rule each tool ships today, from its own docs or source, with its version | the full list, not just the presets W-03 read | ✅ 2026-10-30 — 3,431 rules from 19 packages, read from the packages themselves |
| 2 | Match each one to codelaw's rules | what is already covered | ✅ 689 rules; 653 upstream rules covered (27% of the 2,438 that apply) |
| 3 | Sort every gap into one class (below) and propose a pack, an id and a fix class for each one worth porting | a build list per pack | ✅ 1,690 to port, in tranches A–E |
| 4 | A proposal in `work/proposals/` with the counts and the full table; Arpit reviews it | what W-139 builds | ✅ ruled 2026-10-30 — build all (1,745 with the additions); **closed** |

**Arpit, 2026-10-30 (voice):** on depth inside each tool's own language — *"I want one work item to research all the rules in depth for all the tools and then go ahead and implement it within code law."* → *"One big item for Opus to research it and once the research is done, then one big work item to build it."*

**Researched 2026-10-30 (Cowork, Opus 5.5):** *"Research work item 138, create a list. … Make sure we first have a count of all the rules for ESLint, Ruff, and all the linters … and then put a count."* → [the proposal](../../work/proposals/rule-parity.md), the per-tool CSVs and the scripts in [`rule-parity/`](../../work/proposals/rule-parity/). Two classes added to the eight: `planned` (filed elsewhere) and `same-as` (one rule counted once across tools). Found on the way: the formatter's measured gap → [W-141](../../work/open/W-141-formatter-parity.md).

**Model:** Cowork (Opus 5.5) — research (SR-WORK-OPEN-QUEUE rule 37a): Arpit asked for Opus; Claude Code does not pick research items.

**Ruled 2026-10-30 (Arpit, voice; written on *"file it."*):**

- **R1 — build everything, in one go** (*"I would suggest building out all the rules in single shot."*): all five tranches, 1,690 rules; plus ESLint's 16 `skip` taste rules (*"Let's build those 16."*), in an opt-in preset; plus Ruff's 39 framework rules (*"Let's build the 39 ones."*) as **five new packs** (*"Give each framework its own pack."*): `django`, `fastapi`, `airflow`, `numpy`, `pandas`, PyPI only as L6 ships `python` — names W-142. **1,745 rules.**
- **R2 — as recommended** (*"Go with your recommendation."*): tranche A joins each pack's `recommended` preset (on by default); everything else lands `preview`, in `strict` or opt-in presets; unicorn as its own opt-in preset; the 16 taste rules as their own opt-in preset.
- **R3 — as recommended:** `[format]` keys for `arrowParens`, `bracketSpacing`, `jsxSingleQuote`, `quoteProps`; Prettier's default kept for the other seven — W-141.
- **R4 — W-141 as filed.**
- **Also ruled the same call:** Biome's 89 Vue/Svelte/Astro/GraphQL rules → four new languages, W-143 (after W-139); more linters (sonarjs, test plugins, promise, jsdoc, regexp, full Pylint, Next.js, Tailwind) and TSLint's 30 without a mapped successor → W-144; a Tailwind pack with class sorting in the formatter → W-145.

## Goal

The tools: **ESLint** (core), **typescript-eslint**, **TSLint** (retired 2019 — map its rules to their successors, port none for their own sake), **Ruff** (every linter it carries: pyflakes, pycodestyle, pylint, bugbear, bandit `S`, pydocstyle …), **Prettier** (formatter options and behaviours, not rules), **Biome**, **stylelint** (with stylelint-scss), **html-validate**, **markdownlint**. Also the ESLint plugins codelaw's packs already answer to: react, react-hooks, jsx-a11y, import/n, unicorn, security, @tanstack/query, the Redux Toolkit rules — the sources B-048 names.

Every upstream rule gets exactly one class:

| class | means |
|---|---|
| `covered` | a codelaw rule does it — name it |
| `port` | worth porting — proposed pack, id, fix class (L4), and why it earns its place |
| `types` | needs type information — goes to W-105's type questions |
| `layout` | the formatter owns it (SR-FORMATTER decision 3) — check the formatter does |
| `out-of-scope` | a language outside L3 (Vue, Svelte, Angular, Less …) or a framework codelaw has no pack for |
| `needs-data` | needs data codelaw cannot ship under L2 (browser support, spelling outside SCOWL …) |
| `retired` | deprecated or removed upstream |
| `skip` | judged not worth it — say why in one line |

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · lists | each tool's full rule list, with the version and the date read, kept as data (`work/proposals/rule-parity/<tool>.csv`: upstream id, class, codelaw id, proposed pack, fix class, note) | ✅ |
| 2 · proposal | `work/proposals/rule-parity.md`: counts per tool and per class; the `port` list per pack, ordered by value; the Prettier comparison measured on the corpus, not guessed; what B-048's presets need | ✅ (Prettier: 1,279 files, §7) |
| 3 · ruling | Arpit rules on the port list (all, a cut, or per pack) — W-139 builds that | ✅ all |

## Blockers

None to research. W-139 waits on DoD 3.

## Record affected

**No SR affected** — research. The ruling feeds SR-RULE-PACKS through W-139.

## Hazards

- **Rewrite, never copy** (L2, L11): read upstream behaviour; write codelaw's own rule text, cases and docs.
- Count from the source, not a blog: rule lists change every release — record the version read.
- W-03 already researched the presets; build on its table, do not redo it.
- A rule whose only fix is a guess is `manual`, carried by a skill (L4) — say so in the proposal, not after.

## Reference

[W-03](W-03-rule-packs.md) (the preset research) · B-048 (the later presets) · B-050 (`migrate`, which wants the same mapping) · W-105 (type-aware rules) · [W-139](../../work/open/W-139-rule-parity-build.md)
