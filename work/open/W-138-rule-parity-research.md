---
type: Handoff
name: W-138
description: "Arpit, 2026-10-30 (voice): research every rule of ESLint, typescript-eslint, TSLint, Ruff, Prettier, Biome, stylelint, html-validate and markdownlint against codelaw's packs, and say which to port."
item: W-138
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: cowork
---

# W-138 — Research: every rule of nine tools, matched against codelaw

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | List every rule each tool ships today, from its own docs or source, with its version | the full list, not just the presets W-03 read | ⏳ |
| 2 | Match each one to codelaw's 665 rules | what is already covered | ⏳ |
| 3 | Sort every gap into one class (below) and propose a pack, an id and a fix class for each one worth porting | a build list per pack | ⏳ |
| 4 | A proposal in `work/proposals/` with the counts and the full table; Arpit reviews it | what W-139 builds | 🔴 Arpit reviews |

**Arpit, 2026-10-30 (voice):** on depth inside each tool's own language — *"I want one work item to research all the rules in depth for all the tools and then go ahead and implement it within code law."* → *"One big item for Opus to research it and once the research is done, then one big work item to build it."*

**Model:** Cowork (Opus 5.5) — research (SR-WORK-OPEN-QUEUE rule 37a): Arpit asked for Opus; Claude Code does not pick research items.

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
| 1 · lists | each tool's full rule list, with the version and the date read, kept as data (`work/proposals/rule-parity/<tool>.csv`: upstream id, class, codelaw id, proposed pack, fix class, note) | ⏳ |
| 2 · proposal | `work/proposals/rule-parity.md`: counts per tool and per class; the `port` list per pack, ordered by value; the Prettier comparison measured on the corpus, not guessed; what B-048's presets need | ⏳ |
| 3 · ruling | Arpit rules on the port list (all, a cut, or per pack) — W-139 builds that | 🔴 |

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

[W-03](../../archive/open/W-03-rule-packs.md) (the preset research) · B-048 (the later presets) · B-050 (`migrate`, which wants the same mapping) · W-105 (type-aware rules) · [W-139](W-139-rule-parity-build.md)
