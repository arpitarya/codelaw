---
type: Handoff
name: W-141
description: "The formatter, closer to Prettier where it measurably differs (W-138's corpus: CSS 42%, HTML 17%) — including Oxc's native formatters, measured and put under L11 (was W-148). Merged 2026-10-30."
item: W-141
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-141 — The formatter, closer to Prettier (and Oxc's formatters)

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | CSS: a comment's `*/` and the next selector stay on separate lines | a defect gone | ⏳ |
| 2 | HTML: void elements written as Prettier writes them (`<meta charset="utf-8" />`) | most of the HTML gap | ⏳ |
| 3 | Rerun the measurement; then Markdown code blocks, CSS `calc()` breaks, JS/TS comment placement | each language within 2% of Prettier | ⏳ |
| 4 | The corpus and the comparison kept as a repeatable check | the number cannot slip back unseen | ⏳ |
| W-148 | From Oxlint: Oxc's formatters, 18 rules, 38 bug rules — merged in; its steps under [§ W-148](#merged-in-w-148) | one item, not two | ⏳ — see § |

**Merged 2026-10-30 (Arpit, text):** *"There are too many work items. See if we can review all the work items and merge them into few of them."* → *"go ahead, merge them."* Merged in: W-148.

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

[Proposal](../proposals/rule-parity.md) §7 · `work/proposals/rule-parity/method/` · W-07 (the formatter) · B-045 · [W-148](W-141-formatter-parity.md) (Oxc's formatters measured on this corpus: CSS 88.6%, Markdown 97.3%)

## Merged in: W-148

<a id="merged-in-w-148"></a>

**W-148 — From Oxlint: Oxc's formatters, 18 rules, 38 bug rules**, merged into W-141 on 2026-10-30; its file is archived at [`archive/open/W-148-oxc-formatters-and-oxlint-rules.md`](../../archive/open/W-148-oxc-formatters-and-oxlint-rules.md). As it stood:

### At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Build `oxc_formatter_css`, `_markdown`, `_yaml` (and `oxc_formatter` for JS/TS) behind codelaw's formatter on a branch; rerun W-141's corpus; measure size, speed (L8) and the layout check | a measured case, not a claim | ⏳ |
| 2 | Write the L11 ratification entry (crates, pinned rev, licences, dependency tree, binary size) and put it to Arpit | his sign-off, or a no | ⏳ then 🔴 Arpit |
| 3 | The 18 Oxlint-only rules (`oxlint.csv`, class `port`) built in W-139 | bugs no other tool catches | ✅ added to W-139's list 2026-10-30 |
| 4 | The 38 tranche B/C ports Oxlint ranks `correctness` run on the corpus once W-139 has them; those with no false positives proposed for `recommended` | more bugs caught by default, safely | 🟡 waits on W-139 |

**Arpit, 2026-10-30 (voice):** *"Put it in backlog. Everything else, put it under a work. File it under a work item."* (the backlog part is `codelaw migrate oxlint|oxfmt`, B-066) · then *"File it."*

**Researched 2026-10-30 (Cowork, Opus 5.5):** *"Research auxlint and what can be used from that into code law."* — no tool by that name; read as Oxlint (Oxc, MIT). [Proposal §12](../proposals/rule-parity.md).

**Model:** Opus (Claude Code): a formatter swap measured against a corpus, and a dependency case under L11.

### Goal

**Step 1's starting numbers.** oxfmt 0.75.0 run on W-141's corpus with `printWidth: 80`, against Prettier 3.9.10's output (byte-identical files):

| language | files | oxfmt | codelaw 0.4.0 |
|---|--:|--:|--:|
| CSS | 79 | 88.6% | 41.8% |
| Markdown | 150 | 97.3% | 73.3% |
| YAML | 150 | 97.3% | 88.7% |
| SCSS | 150 | 88.0% | 82.7% |
| JS | 150 | 96.0% | 94.7% |
| TS | 150 | 94.0% | 94.0% |

- **Native Rust, MIT:** CSS/SCSS/Less, Markdown, YAML, JSON, TOML, GraphQL and JS/TS (`apps/oxfmt/src/core/support.rs`, `FormatStrategy::Native`). Their direct dependencies are small (`oxc-css-parser`, `oxc-markdown-parser`, `oxc-yaml-parser`, `oxc_formatter_core`, `oxc_allocator`).
- **Not on crates.io** (`publish = false`): a git-rev dependency, as codelaw pins Biome's today.
- **Not usable:** HTML, Vue, Svelte, Astro — oxfmt hands them to Prettier (JavaScript); oxfmt's 100% on HTML is Prettier's own output. W-143 cannot take them.
- **JS/TS gain is small** (+1.3 points JS): the case is CSS and Markdown; decide JS/TS separately.
- Oxc's own Prettier-fixture numbers (`apps/oxfmt/conformance/snapshots/conformance.snap.md`): CSS 221/221, Markdown 273/283, YAML 301/302, SCSS 203/217, GraphQL 712/712.

**Step 3's rules** (11 in `js`/`node`, 7 in `react`): array methods on `arguments`; `charAt` compared with a multi-character string; `matchAll`/`replaceAll` given a regex without `g`; `if`/`else` sharing code; erasing ops (`x * 0`); `async` Express handlers; spreads inside `.map`; `this` in an exported function; out-of-range radix/precision; callbacks on arrays of empty slots; `cloneElement`; `React.Children`; `only-export-components` (Fast Refresh); class components; JSX, array and object literals passed as props.

**Step 4's list:** `oxlint.csv` rows whose `oxlint_category` is `correctness` and whose `same-as` source row is tranche B or C — 38 (`js/no-eval`, `js/no-thenable`, `react/no-unsafe`, `test/no-conditional-tests` …).

### Definition of done

| DoD | what | state |
|---|---|---|
| 1 · measured | corpus table per language for codelaw-with-Oxc; L8 budgets and the layout check unchanged or explained; binary size before/after | ⏳ |
| 2 · ruled | an L11 entry (D-nn) ratified or refused by Arpit; if ratified, SR-FORMATTER names which languages use which formatter and W-141 continues from there | ⏳ |
| 3 · rules | the 18 in W-139's DoD 1 | ✅ |
| 4 · defaults | the 38 corpus-run; promotions proposed with their finding counts | ⏳ |

### Blockers

None for steps 1–2. Step 4 🟡 W-139.

### Record affected

[SR-LAW-11](../../records/0013_LAW-11-dependencies.md) (a ratified dependency) · [SR-FORMATTER](../../records/0105_formatter.md) · SR-RULE-PACKS (presets, step 4).

### Hazards

- **Output churn:** a formatter swap reformats consumers' files on upgrade; CHANGELOG it, and keep each language's switch separate.
- **The layout check** (SR-FORMATTER decision 3) must still see the same tokens.
- **A git rev on an unpublished crate** moves only when codelaw moves it; pin it, and say who bumps it.
- **Rules:** reimplement from behaviour; Oxlint is MIT, but codelaw writes its own cases and docs.

### Reference

[W-141](W-141-formatter-parity.md) · [W-139](W-139-rule-parity-build.md) · [W-143](W-143-vue-svelte-astro-graphql.md) · `work/proposals/rule-parity/oxlint.csv` · `method/oxlint-rules.jsonl` · [oxc-project/oxc](https://github.com/oxc-project/oxc) (MIT)
