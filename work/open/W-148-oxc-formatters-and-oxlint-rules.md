---
type: Handoff
name: W-148
description: "Arpit, 2026-10-30 (voice): from Oxlint/Oxc — test Oxc's native formatters against codelaw's for W-141 and bring them for L11 sign-off; 18 Oxlint-only rules join W-139; 38 bug rules Oxlint ranks correctness are corpus-run before any joins recommended."
item: W-148
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-148 — From Oxlint: Oxc's formatters, 18 rules, 38 bug rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Build `oxc_formatter_css`, `_markdown`, `_yaml` (and `oxc_formatter` for JS/TS) behind codelaw's formatter on a branch; rerun W-141's corpus; measure size, speed (L8) and the layout check | a measured case, not a claim | ⏳ |
| 2 | Write the L11 ratification entry (crates, pinned rev, licences, dependency tree, binary size) and put it to Arpit | his sign-off, or a no | ⏳ then 🔴 Arpit |
| 3 | The 18 Oxlint-only rules (`oxlint.csv`, class `port`) built in W-139 | bugs no other tool catches | ✅ added to W-139's list 2026-10-30 |
| 4 | The 38 tranche B/C ports Oxlint ranks `correctness` run on the corpus once W-139 has them; those with no false positives proposed for `recommended` | more bugs caught by default, safely | 🟡 waits on W-139 |

**Arpit, 2026-10-30 (voice):** *"Put it in backlog. Everything else, put it under a work. File it under a work item."* (the backlog part is `codelaw migrate oxlint|oxfmt`, B-066) · then *"File it."*

**Researched 2026-10-30 (Cowork, Opus 5.5):** *"Research auxlint and what can be used from that into code law."* — no tool by that name; read as Oxlint (Oxc, MIT). [Proposal §12](../proposals/rule-parity.md).

**Model:** Opus (Claude Code): a formatter swap measured against a corpus, and a dependency case under L11.

## Goal

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

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · measured | corpus table per language for codelaw-with-Oxc; L8 budgets and the layout check unchanged or explained; binary size before/after | ⏳ |
| 2 · ruled | an L11 entry (D-nn) ratified or refused by Arpit; if ratified, SR-FORMATTER names which languages use which formatter and W-141 continues from there | ⏳ |
| 3 · rules | the 18 in W-139's DoD 1 | ✅ |
| 4 · defaults | the 38 corpus-run; promotions proposed with their finding counts | ⏳ |

## Blockers

None for steps 1–2. Step 4 🟡 W-139.

## Record affected

[SR-LAW-11](../../records/0013_LAW-11-dependencies.md) (a ratified dependency) · [SR-FORMATTER](../../records/0105_formatter.md) · SR-RULE-PACKS (presets, step 4).

## Hazards

- **Output churn:** a formatter swap reformats consumers' files on upgrade; CHANGELOG it, and keep each language's switch separate.
- **The layout check** (SR-FORMATTER decision 3) must still see the same tokens.
- **A git rev on an unpublished crate** moves only when codelaw moves it; pin it, and say who bumps it.
- **Rules:** reimplement from behaviour; Oxlint is MIT, but codelaw writes its own cases and docs.

## Reference

[W-141](W-141-formatter-parity.md) · [W-139](W-139-rule-parity-build.md) · [W-143](W-143-vue-svelte-astro-graphql.md) · `work/proposals/rule-parity/oxlint.csv` · `method/oxlint-rules.jsonl` · [oxc-project/oxc](https://github.com/oxc-project/oxc) (MIT)
