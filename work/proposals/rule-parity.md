---
type: Proposal
title: Rule parity — every rule of nine linters, counted and matched against codelaw
description: "W-138's outcome: 3,431 upstream rules from 19 packages (ESLint, typescript-eslint, TSLint, Ruff, Biome, stylelint, html-validate, markdownlint and the ESLint plugins codelaw's packs answer to) and Prettier's 29 options, each matched against codelaw's 689 rules and classed; 1,690 worth porting, in five tranches; the formatter measured against Prettier and Ruff on 1,279 files."
status: proposed
timestamp: 2026-10-30T02:45:00+05:30
---

# Rule parity — every rule, counted

**For the decider to rule (§9).** Researched 2026-10-30 (Cowork, Opus 5.5) for
[W-138](../open/W-138-rule-parity-research.md); [W-139](../open/W-139-rule-parity-build.md)
builds what is ruled. Every number here is computed by the scripts in
[`rule-parity/method/`](rule-parity/method/) from each tool's own published
package — nothing is counted from a blog or a docs page.

*"Make sure we first have a count of all the rules for ESLint, Ruff, and all the linters. And then we go ahead and make a list of them on how code law is going to work, and then put a count."* — Arpit, 2026-10-30

## 1 · In one line

**The nine linters ship 3,431 rules; after setting aside the retired (317),
the formatting rules the formatter owns (230), other languages (128) and the
duplicates between tools (318), 2,438 apply to codelaw — and codelaw covers 653
of them (27%). 1,690 are worth porting; done in full, codelaw goes from 689 to
about 2,380 rules.**

## 2 · How the lists were read

| tool | package and version | how its rules were read |
|---|---|---|
| ESLint | `eslint` 10.13.0 | `builtinRules`, each rule's `meta` |
| typescript-eslint | `@typescript-eslint/eslint-plugin` 8.72.1 | `rules[*].meta` (`requiresTypeChecking`, `extendsBaseRule`) |
| ESLint plugins | react 7.37.5, react-hooks 7.1.1, jsx-a11y 6.10.2, import 2.32.0, import-x 4.17.1, n 18.5.0, unicorn 79.0.0, security 4.2.0, @tanstack/query 5.105.0, @stylistic 5.10.0 | `rules[*].meta` |
| TSLint | `tslint` 6.1.3 (retired 2019) | `lib/rules/*Rule.js` metadata; successors from `tslint-to-eslint-config` |
| Ruff | `ruff` 0.16.9 | `ruff rule --all --output-format json` |
| Biome | `@biomejs/biome` 2.6.4 | the configuration schema; `biome explain` per rule; sources from Biome's own `declare_lint_rule!` blocks at the 2.6.4 tag |
| stylelint | `stylelint` 17.16.0, `stylelint-scss` 7.4.0 | `rules[*].meta` |
| html-validate | `html-validate` 11.17.0 | `builtinRules` |
| markdownlint | `markdownlint` 0.42.0 | the library's rule list |
| Prettier | `prettier` 3.9.10 | `getSupportInfo().options` |

"Recommended" is each tool's own preset (`@eslint/js` recommended, each plugin's
`recommended`, `stylelint-config-recommended[-scss]`,
`html-validate:recommended`, Ruff's default `E4, E7, E9, F`, Biome's
recommended set, markdownlint's default all-on). codelaw's side is every
`packs/*/rules/**/*.md` on `main` at 2026-10-30, matched through `ported_from:`.

## 3 · The classes

| class | means |
|---|---|
| **covered** | a codelaw rule does it (`ported_from:` names it; `*` = an equivalent that does not name it yet — 29, W-139 adds the line) |
| **port** | worth porting — proposed pack, id and fix class in the CSV |
| **types** | needs type information — W-105's type questions |
| **layout** | formatting; the formatter owns it (SR-FORMATTER decision 3) |
| **planned** | already filed as another work item |
| **needs-data** | needs runtime-support data codelaw does not ship (`n/no-unsupported-features/*`) |
| **out-of-scope** | a language outside L3 (Vue, Svelte, Astro, GraphQL) or a framework with no codelaw pack (Ruff's Airflow, Django, FastAPI, NumPy, pandas rules) |
| **same-as** | the same rule as one counted under another tool (Biome's sourced rules, import-x's fork, typescript-eslint's extension rules) — counted once |
| **skip** | taste with no defect behind it, or about the source tool's own config or comments — portable on demand through W-140 |
| **retired** | deprecated or removed upstream; all of TSLint |

## 4 · The counts, tool by tool

"Applicable" = rules − retired − layout − out-of-scope − same-as. "Covers" = covered ÷ applicable.

| tool | version | rules | covered | port | types | layout | planned | needs-data | out-of-scope | same-as | skip | retired | applicable | codelaw covers |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| ruff | 0.16.9 | 970 | 175 | 674 |  | 59 | 7 |  | 39 |  | 1 | 15 | 857 | 20% |
| biome | 2.6.4 | 677 | 135 | 178 |  |  |  |  | 89 | 264 | 3 | 8 | 316 | 43% |
| eslint-plugin-unicorn | 79.0.0 | 396 | 1 | 366 |  | 9 |  |  |  |  |  | 20 | 367 | 0% |
| eslint | 10.13.0 | 292 | 81 | 101 |  | 1 |  |  |  |  | 16 | 93 | 198 | 41% |
| tslint | 6.1.3 | 163 |  |  |  |  |  |  |  |  |  | 163 | 0 | — |
| stylelint | 17.16.0 | 150 | 71 | 79 |  |  |  |  |  |  |  |  | 150 | 47% |
| typescript-eslint | 8.72.1 | 136 | 30 | 30 | 58 |  |  |  |  | 9 |  | 9 | 118 | 25% |
| eslint-plugin-react | 7.37.5 | 103 | 19 | 78 |  | 1 |  |  |  |  | 3 | 2 | 100 | 19% |
| @stylistic/eslint-plugin | 5.10.0 | 97 |  |  |  | 97 |  |  |  |  |  |  | 0 | — |
| html-validate | 11.17.0 | 94 | 70 | 11 |  | 11 |  |  |  |  | 2 |  | 83 | 84% |
| stylelint-scss | 7.4.0 | 71 | 10 | 40 |  | 21 |  |  |  |  |  |  | 50 | 20% |
| markdownlint | 0.42.0 | 53 | 17 | 5 |  | 31 |  |  |  |  |  |  | 22 | 77% |
| eslint-plugin-import-x | 4.17.1 | 47 |  | 2 |  |  |  |  |  | 45 |  |  | 2 | 0% |
| eslint-plugin-import | 2.32.0 | 46 | 3 | 42 |  |  |  |  |  |  |  | 1 | 45 | 7% |
| eslint-plugin-n | 18.5.0 | 45 | 10 | 30 |  |  |  | 3 |  |  |  | 2 | 43 | 23% |
| eslint-plugin-jsx-a11y | 6.10.2 | 39 |  | 36 |  |  |  |  |  |  |  | 3 | 36 | 0% |
| eslint-plugin-react-hooks | 7.1.1 | 29 | 11 | 15 |  |  |  |  |  |  | 2 | 1 | 28 | 39% |
| eslint-plugin-security | 4.2.0 | 15 | 13 | 2 |  |  |  |  |  |  |  |  | 15 | 87% |
| @tanstack/eslint-plugin-query | 5.105.0 | 8 | 7 | 1 |  |  |  |  |  |  |  |  | 8 | 88% |
| **all** | | **3431** | **653** | **1690** | **58** | **230** | **7** | **3** | **128** | **318** | **27** | **317** | **2438** | **27%** |

**Prettier** (options, not rules): 29 options — 8 covered by `[format]` keys, 11
style options codelaw has no key for (`arrowParens`, `bracketSameLine`,
`bracketSpacing`, `experimentalOperatorPosition`, `experimentalTernaries`,
`htmlWhitespaceSensitivity`, `jsxSingleQuote`, `objectWrap`, `proseWrap`,
`quoteProps`, `singleAttributePerLine`), 9 API or CLI arguments, 1 Vue-only.

## 5 · codelaw, pack by pack, before and after

The port list falls into five tranches:

| tranche | what | rules |
|---|---|--:|
| **A** | in the upstream tool's own recommended preset (unicorn and Ruff's long tail apart) | 84 |
| **B** | eslint-plugin-unicorn — 338 of its 396 rules are in its recommended preset; taste-heavy, B-048's `js/strict` | 366 |
| **C** | stable, not recommended upstream: ESLint 101, Biome 100, stylelint 79, react 77, import 34, stylelint-scss 33, typescript-eslint 30, n 29, react-hooks 11, html-validate 11, jsx-a11y 5, import-x 2, query 1 | 513 |
| **D** | Ruff beyond its default set: Pylint 93, flake8-pyi 55, Ruff-specific 50, pydocstyle 46, pyupgrade 45, flake8-use-pathlib 35, flake8-simplify 30, flake8-pytest-style 28, refurb 22, flake8-comprehensions 19, and 31 smaller linters | 591 |
| **E** | preview upstream (Ruff preview 83, Biome nursery 53) | 136 |

| pack | codelaw today | + A | + B | + C | + D | + E | after all |
|---|--:|--:|--:|--:|--:|--:|--:|
| css | 73 |  |  | 81 |  | 5 | 159 |
| css-modules | 5 |  |  |  |  |  | 5 |
| data | 44 | 5 |  | 5 |  | 4 | 58 |
| html | 74 | 1 |  | 11 |  |  | 86 |
| js | 84 | 25 | 366 | 200 |  | 25 | 700 |
| node | 10 | 1 |  | 29 |  |  | 40 |
| python | 109 |  |  |  | 589 | 80 | 778 |
| react | 35 | 40 |  | 121 |  | 15 | 211 |
| redux-toolkit | 12 |  |  |  |  |  | 12 |
| scss | 16 | 7 |  | 33 |  |  | 56 |
| security-html | 19 |  |  |  |  |  | 19 |
| security-js | 66 | 2 |  |  |  |  | 68 |
| security-python | 61 |  |  |  | 2 | 3 | 66 |
| security-ts | 16 |  |  |  |  |  | 16 |
| spell | 7 |  |  |  |  |  | 7 |
| stencil | 18 |  |  |  |  |  | 18 |
| tanstack-query | 13 |  |  | 1 |  |  | 14 |
| ts | 27 | 3 |  | 32 |  | 4 | 66 |
| **all** | **689** | **84** | **366** | **513** | **591** | **136** | **2379** |

Fix classes proposed from the upstream fix: **538 auto, 319 suggest, 833 manual**
(L4: every `manual` one needs its skill steps). Each is a proposal; W-139 judges
each rule's own fix.

## 6 · Tranche A, rule by rule

- **biome** (25): `a11y/noSvgWithoutTitle`, `complexity/noEmptyTypeParameters`, `complexity/noFlatMapIdentity`, `complexity/noThisInStatic`, `complexity/noUselessContinue`, `complexity/noUselessStringRaw`, `complexity/useSimpleNumberKeys`, `correctness/noConstantMathMinMaxClamp`, `correctness/noDuplicateAttributes`, `correctness/noStringCaseMismatch`, `correctness/noUnusedFunctionParameters`, `correctness/noUnusedImports`, `correctness/noVoidTypeReturn`, `performance/noAccumulatingSpread`, `performance/noDynamicNamespaceImportAccess`, `suspicious/noConstEnum`, `suspicious/noDocumentCookie`, `suspicious/noGlobalIsFinite`, `suspicious/noGlobalIsNan`, `suspicious/noImplicitAnyLet`, `suspicious/noMisrefactoredShorthandAssign`, `suspicious/noRedundantUseStrict`, `suspicious/noSuspiciousSemicolonInJsx`, `suspicious/noUselessEscapeInString`, `suspicious/useGoogleFontDisplay`
- **eslint-plugin-import** (8): `default`, `export`, `named`, `namespace`, `no-duplicates`, `no-named-as-default`, `no-named-as-default-member`, `no-unresolved`
- **eslint-plugin-jsx-a11y** (31): `alt-text`, `anchor-has-content`, `anchor-is-valid`, `aria-activedescendant-has-tabindex`, `aria-props`, `aria-proptypes`, `aria-role`, `aria-unsupported-elements`, `autocomplete-valid`, `click-events-have-key-events`, `heading-has-content`, `html-has-lang`, `iframe-has-title`, `img-redundant-alt`, `interactive-supports-focus`, `label-has-associated-control`, `media-has-caption`, `mouse-events-have-key-events`, `no-access-key`, `no-autofocus`, `no-distracting-elements`, `no-interactive-element-to-noninteractive-role`, `no-noninteractive-element-interactions`, `no-noninteractive-element-to-interactive-role`, `no-noninteractive-tabindex`, `no-redundant-roles`, `no-static-element-interactions`, `role-has-required-aria-props`, `role-supports-aria-props`, `scope`, `tabindex-no-positive`
- **eslint-plugin-n** (1): `process-exit-as-throw`
- **eslint-plugin-react** (1): `prop-types`
- **eslint-plugin-react-hooks** (4): `immutability`, `incompatible-library`, `preserve-manual-memoization`, `unsupported-syntax`
- **eslint-plugin-security** (2): `detect-disable-mustache-escape`, `detect-invisible-characters`
- **markdownlint** (5): `MD026`, `MD028`, `MD043`, `MD044`, `MD054`
- **stylelint-scss** (7): `scss/comment-no-empty`, `scss/declaration-nested-properties-no-divided-groups`, `scss/dollar-variable-no-missing-interpolation`, `scss/function-quote-no-quoted-strings-inside`, `scss/function-unquote-no-unquoted-strings-inside`, `scss/load-no-partial-leading-underscore`, `scss/load-partial-extension`

The full list of every tranche, with proposed ids, is in the CSVs (`tranche` column).

## 7 · The formatter, measured

`codelaw format` (0.4.0) against Prettier 3.9.10 (Ruff 0.16.9 for Python), default
settings both sides, on 1,279 real files: ESLint's own rule sources (JS), typescript-eslint's
declarations (TS), Bootstrap, Bulma and Primer CSS and SCSS, GitHub's starter workflows
(YAML), MDN's learning-area pages (HTML), package READMEs (Markdown) and manifests (JSON),
Debian's Python packages. Byte-identical outputs:

| language | files | identical | |
|---|--:|--:|---|
| JSON | 150 | 150 | 100% |
| Python (against Ruff) | 150 | 150 | 100% |
| JavaScript | 150 | 142 | 95% |
| TypeScript | 150 | 141 | 94% |
| YAML | 150 | 133 | 89% |
| SCSS | 150 | 124 | 83% |
| Markdown | 150 | 110 | 73% |
| CSS | 79 | 33 | 42% |
| HTML | 150 | 26 | 17% |

What the differences are, from the samples read:

- **CSS — a defect:** a comment's closing `*/` and the next selector are joined on one line (`*/ :root,`); `calc()` sums break at different points.
- **HTML — one choice explains most of it:** codelaw writes void elements as `<meta charset="utf-8">`, Prettier as `<meta charset="utf-8" />`. SR-FORMATTER decision 5 says the defaults are Prettier's, so this is a gap, not a choice.
- **Markdown:** embedded code blocks are wrapped where Prettier keeps them (`new Cacheable({/* options */})`).
- **JS, TS:** comment placement inside calls and objects.

## 8 · Other findings

- **typescript-eslint 8.72 refuses TypeScript 7.0** at load ("typescript-eslint does not support TS 7.0") — the reading here used TypeScript 6. It confirms W-105's route: codelaw's own helper, not typescript-eslint.
- **29 codelaw rules do a ported rule's job without naming it** in `ported_from:` (for example `py/no-print` = Ruff `T201`, `ts/type-only-imports` = `consistent-type-imports`, `js/todo-marker` = `no-warning-comments`). `codelaw migrate` cannot map them until they do.
- **3 proposed ids collide** with existing codelaw rules (`js/no-loss-of-precision`, `js/prefer-spread` from unicorn; `scss/load-no-partial-leading-underscore`) — check before porting.
- **The ESLint core gap is not in `recommended`:** every one of ESLint's 64 recommended rules that applies is covered; the 101 to port are its optional rules.
- **Biome is mostly other tools' rules:** 614 of its 663 rules name a source; 264 of them duplicate rules counted here.

## 9 · For the decider

- **R1 — which tranches W-139 builds.** Recommended: **A** (84) and **C** (513) now; **D** (591) by linter, pydocstyle as an opt-in preset; **B** (unicorn, 366) only as B-048's opt-in `js/strict`; **E** (136) waits until upstream calls them stable.
- **R2 — where new rules go.** Recommended: tranche A joins each pack's `recommended` preset (as its source recommends); C and D land `preview`, in `strict` presets or off; nothing new is on by default without a corpus run (SR-RULE-AUTHORING decision 6).
- **R3 — Prettier's 11 options.** Add `[format]` keys for the four people set most (`arrowParens`, `bracketSpacing`, `jsxSingleQuote`, `quoteProps`), and keep Prettier's default for the rest — or keys for all eleven.
- **R4 — the formatter gaps (§7).** Filed as [W-141](../open/W-141-formatter-parity.md): the CSS comment defect and HTML void elements first, then Markdown, CSS, JS/TS until each language is within 2% of Prettier on this corpus.

## 10 · What is in `rule-parity/`

- one CSV per tool — `upstream_id, class, codelaw, proposed_pack, proposed_id, proposed_fix, priority, tranche, upstream_fix, upstream_recommended, deprecated, note` — and `prettier.csv`;
- `summary.json` (the counts above), `ports.json` (ports by pack and priority);
- `method/`: the scripts and their inputs, so the tables can be rebuilt on a newer release of any tool: `node extract.mjs <dir> up.jsonl`, `node rec.mjs <dir> recommended.json`, `python3 classify.py`, `python3 tables.py`.

Licences: names, ids, counts and short notes only — no rule text, docs or test is copied (L2).
