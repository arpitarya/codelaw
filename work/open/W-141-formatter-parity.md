---
type: Handoff
name: W-141
description: "W-138's research, 2026-10-30: codelaw format measured against Prettier on 1,279 real files — CSS 42% and HTML 17% identical; a CSS comment defect and HTML void elements first."
item: W-141
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-141 — The formatter, closer to Prettier where it measurably differs

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | CSS: a comment's `*/` and the next selector stay on separate lines | a defect gone | ⏳ |
| 2 | HTML: void elements written as Prettier writes them (`<meta charset="utf-8" />`) | most of the HTML gap | ⏳ |
| 3 | Rerun the measurement; then Markdown code blocks, CSS `calc()` breaks, JS/TS comment placement | each language within 2% of Prettier | ⏳ |
| 4 | The corpus and the comparison kept as a repeatable check | the number cannot slip back unseen | ⏳ |

**Source:** [W-138's proposal](../proposals/rule-parity.md) §7 — `codelaw format` 0.4.0 against Prettier 3.9.10 (Ruff 0.16.9 for Python), default settings, byte-identical outputs: JSON 100%, Python 100%, JS 95%, TS 94%, YAML 89%, SCSS 83%, Markdown 73%, **CSS 42%, HTML 17%**.

**Model:** Opus (Claude Code): formatter changes in two printers, each checked against the corpus.

## Goal

SR-FORMATTER decision 5 says the defaults are Prettier's (Ruff's for Python). Where codelaw's output differs from Prettier's on real files, codelaw changes — except where a record says otherwise, and then the record says why.

Seen in the samples:

- **CSS** — `/* … */` followed by a selector prints as `*/ :root,` on one line (Primer CSS, `006.css` in the corpus); `calc(a + b)` sums break at different operators.
- **HTML** — `<meta …>`, `<input …>` and every void element: codelaw omits ` /`, Prettier writes it.
- **Markdown** — a fenced JS block's object is wrapped (`new Cacheable({` / `/* options */` / `})`) where Prettier keeps one line.
- **JS / TS** — comments inside calls and object literals placed differently.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · CSS defect | the comment/selector join fixed, with a case | ⏳ |
| 2 · HTML void | Prettier's ` />`, or a record ruling for codelaw's form | ⏳ |
| 3 · the rest | Markdown, CSS, SCSS, YAML, JS, TS each ≥ 98% identical on the corpus, or the difference named in SR-FORMATTER | ⏳ |
| 3b · Prettier keys | `[format]` keys for `arrowParens`, `bracketSpacing`, `jsxSingleQuote`, `quoteProps` (W-138 R3, ruled 2026-10-30); the other seven Prettier options stay at Prettier's default | ⏳ |
| 4 · repeatable | the corpus (sources and licences listed, not vendored where large) and a script that prints the table — in `perf/` or `fixtures/`; not in CI unless cheap | ⏳ |

## Blockers

None.

## Record affected

[SR-FORMATTER](../../records/0105_formatter.md) decisions 3 and 5.

## Hazards

- A change to output reformats every consumer's files on upgrade — CHANGELOG it as a formatting change.
- The layout check (SR-FORMATTER decision 3) compares token streams: none of these changes may alter tokens.
- Corpus files keep their licences (Bootstrap MIT, Bulma MIT, Primer MIT, starter-workflows MIT, MDN learning-area CC0/MIT); name them, do not vendor large trees.

## Reference

[Proposal](../proposals/rule-parity.md) §7 · `work/proposals/rule-parity/method/` · W-07 (the formatter) · B-045 · [W-148](W-148-oxc-formatters-and-oxlint-rules.md) (Oxc's formatters measured on this corpus: CSS 88.6%, Markdown 97.3%)
