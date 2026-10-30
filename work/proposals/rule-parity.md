---
type: Proposal
title: Rule parity — every rule of nine linters, counted and matched against codelaw
description: "W-138's and W-144's outcome (round 2: SonarJS, Jest, Vitest, Testing Library, Playwright, promise, jsdoc, regexp, Next.js, two Tailwind plugins, Pylint — 1,211 more rules, 879 more to port). Round 1: 3,431 upstream rules from 19 packages (ESLint, typescript-eslint, TSLint, Ruff, Biome, stylelint, html-validate, markdownlint and the ESLint plugins codelaw's packs answer to) and Prettier's 29 options, each matched against codelaw's 689 rules and classed; 1,690 worth porting, in five tranches; the formatter measured against Prettier and Ruff on 1,279 files."
status: graduated
timestamp: 2026-10-30T02:45:00+05:30
---

# Rule parity — every rule, counted

**Ruled 2026-10-30 (Arpit, voice):** build all — 1,690 + 16 taste (opt-in) + 39 framework rules in five new PyPI packs = **1,745** (W-139); tranche A on by default, the rest preview/opt-in; four Prettier keys (W-141); Vue/Svelte/Astro/GraphQL as languages (W-143); round 2 (W-144); Tailwind (W-145). W-138 closed. **Round 2 ruled 2026-10-30 (voice):** a `testing` and a `nextjs` npm pack; SonarJS, Pylint and jsdoc opt-in; regexp and promise on by default; TSLint's 6 built — W-139 (§11); W-144 closed. Researched 2026-10-30 (Cowork, Opus 5.5) for
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


## 11 · Round 2 (W-144, researched 2026-10-30)

*"Research w-144"* — Arpit, 2026-10-30. Same method, same scripts (`method/`, extended; `pylint_rules.py` added). Each tool read from its own package:

| tool | version | licence | rules | covered | port | types | layout | same-as | other | notes |
|---|---|---|--:|--:|--:|--:|--:|--:|--:|---|
| eslint-plugin-sonarjs | 4.2.2 | LGPL-3.0 | 295 | 15 | 201 | 59 | | 7 | 13 retired | 7 repackage ESLint/typescript-eslint/React/regexp rules; behaviour only (L2) |
| eslint-plugin-jest | 29.17.0 | MIT | 71 | 2 | 64 | 4 | 1 | | | |
| @vitest/eslint-plugin | 1.6.28 | MIT | 82 | 2 | 25 | | | 55 | | 55 share Jest's rule — counted there |
| eslint-plugin-testing-library | 7.16.2 | MIT | 29 | | 29 | | | | | |
| eslint-plugin-playwright | 2.12.1 | MIT | 67 | 2 | 64 | | 1 | | | |
| eslint-plugin-promise | 7.3.0 | ISC | 17 | | 17 | | | | | |
| eslint-plugin-jsdoc | 65.4.2 | BSD-3 | 78 | | 71 | | 7 | | | an opt-in preset, as pydocstyle |
| eslint-plugin-regexp | 3.3.2 | MIT | 82 | | 80 | | 2 | | | |
| @next/eslint-plugin-next | 16.4.3 | MIT | 22 | | 22 | | | | | a `nextjs` pack |
| eslint-plugin-tailwindcss | 4.4.0 | MIT | 9 | | 4 | | 1 | 4 | | 4 the same checks as better-tailwindcss |
| eslint-plugin-better-tailwindcss | 4.10.0 | MIT | 15 | | 4 | | 11 | | | 11 are class order and spacing — the formatter, W-145 |
| Pylint (with its 51 extension messages) | 4.1.3 | GPL-2.0 | 444 | 14 | 298 | 30 | 8 | 80 | 14 skip | 80 already in Ruff; 30 need Python type inference codelaw has no route for; 14 are run messages; behaviour only (L2) |
| **round 2** | | | **1,211** | **35** | **879** | **93** | **31** | **146** | **27** | |

**TSLint's 30 without a mapped successor** (`tslint-recheck.csv`): 12 now have one elsewhere (counted there), 2 are covered, 5 are formatting, 5 need types, **6 are worth building** — `js/file-header` (licence/copyright header), `js/no-default-import`, `ts/no-mergeable-namespace`, `js/no-unnecessary-callback-wrapper` (suggest only), and `js/comment-type`, `js/switch-final-break` in the taste preset.

**Biome, recounted:** 52 Biome rules whose source is now read (SonarJS, Jest, Vitest, regexp, jsdoc, promise, Next.js, Playwright, Tailwind) move from Biome's `port` to `same-as` — counted under their source. Round 1's 1,745 is now 1,693 under round-1 tools plus those 52 under round-2 tools.

**The CSVs now carry the 2026-10-30 rulings:** ESLint's 16 taste rules are `port` (P3, the opt-in taste preset); Ruff's 39 framework rules are `port` in `django`, `fastapi`, `airflow`, `numpy`, `pandas`; Biome's 89 note W-143.

### Both rounds, pack by pack

| pack | codelaw today | + A | + B | + C | + D | + E | after all |
|---|--:|--:|--:|--:|--:|--:|--:|
| airflow | 0 |  |  |  | 7 | 6 | 13 |
| css | 73 |  |  | 81 |  | 5 | 159 |
| css-modules | 5 |  |  |  |  |  | 5 |
| data | 44 | 5 |  | 5 |  | 4 | 58 |
| django | 0 |  |  |  | 7 |  | 7 |
| fastapi | 0 |  |  |  | 3 |  | 3 |
| html | 74 | 1 |  | 11 |  |  | 86 |
| js | 84 | 296 | 365 | 271 |  | 34 | 1050 |
| nextjs | 0 | 22 |  |  |  |  | 22 |
| node | 10 | 1 |  | 29 |  |  | 40 |
| numpy | 0 |  |  |  | 4 |  | 4 |
| pandas | 0 |  |  |  | 12 |  | 12 |
| python | 109 | 77 |  |  | 810 | 80 | 1076 |
| react | 35 | 39 |  | 111 |  | 11 | 196 |
| redux-toolkit | 12 |  |  |  |  |  | 12 |
| scss | 16 | 7 |  | 33 |  |  | 56 |
| security-html | 19 |  |  |  |  |  | 19 |
| security-js | 66 | 2 |  |  |  |  | 68 |
| security-python | 61 |  |  |  | 2 | 3 | 66 |
| security-ts | 16 |  |  |  |  |  | 16 |
| spell | 7 |  |  |  |  |  | 7 |
| stencil | 18 |  |  |  |  |  | 18 |
| tailwind | 0 | 6 |  | 2 |  |  | 8 |
| tanstack-query | 13 |  |  | 1 |  |  | 14 |
| testing | 0 | 76 |  | 106 |  |  | 182 |
| ts | 27 | 3 |  | 32 |  | 4 | 66 |
| **all** | **689** | **535** | **365** | **682** | **845** | **147** | **3263** |

Tranches: **A** recommended upstream · **B** unicorn · **C** stable, not recommended · **D** Ruff's and Pylint's long tail · **E** preview upstream and the taste preset. **2,574 to port** (+6 from TSLint's recheck = **2,580**): 705 auto, 370 suggest, 1,499 manual. 44 codelaw rules do an upstream rule's job without naming it (`*` in the CSVs).

### Round 2, for the decider

- **R5 — a `testing` pack** (npm only: `@codelaw/rules-testing`): Jest, Vitest, Testing Library and Playwright, 182 rules, a preset per framework (`testing/jest`, `testing/vitest`, `testing/playwright`, `testing/testing-library`). Python's test rules stay in `python` (Ruff's `PT`). Alternative: four packs. L6 amended either way.
- **R6 — a `nextjs` pack** (npm only: `@codelaw/rules-nextjs`), 22 rules, `nextjs/recommended` on when installed.
- **R7 — where round 2 goes by default.** Recommended: each source becomes an **opt-in preset** mirroring its own recommended set (`js/sonarjs`, `js/jsdoc`, `js/regexp`, `js/promise`, `py/pylint`) — SonarJS alone recommends 230 rules, too many to switch on for everyone; `nextjs` and `tailwind` presets on when their pack is installed; nothing joins `js/recommended` or `py/recommended` without a corpus run.
- **R8 — TSLint's 6.** Recommended: build them (4 in their packs, 2 in the taste preset).

The ports join W-139's build (ruled 2026-10-30: *"built along with everything else"*) once R5–R7 say where they live.

**Round 2 ruled 2026-10-30 (Arpit, voice; written on *"file it."*):**

- **R5 — a `testing` pack** (*"The testing pack."* → *"Let's create it."*): one npm pack, `@codelaw/rules-testing`, 182 rules for Jest, Vitest, Testing Library and Playwright, with a preset per framework (`testing/jest`, `testing/vitest`, `testing/testing-library`, `testing/playwright`) so a project switches on only the frameworks it uses. Python's test rules stay in `python` (Ruff's `PT`). L6 amended.
- **R6 — a `nextjs` pack** (*"The next JS pack. Let's create it."*): npm only, `@codelaw/rules-nextjs`, 22 rules; `nextjs/recommended` on when the pack is installed. L6 amended.
- **R7 — what is on by default:** SonarJS and Pylint **off** (*"Off by default. For Sonar.js or Pylint."*), as opt-in presets inside their packs, not new packs (told so → *"Perfect."*): `js/sonarjs` (201) and `py/pylint` (298). jsdoc off too, `js/jsdoc` (71), as recommended. regexp and promise **on** (*"Yes."*): their 70 upstream-recommended rules (regexp 60, promise 10) join `js/recommended`; their other 27 land `preview`, as R2. Each opt-in preset mirrors its source's recommended set; the testing presets likewise; `nextjs` and `tailwind` on when their pack is installed.
- **R8 — TSLint's 6, built** (no new package; *"Let's file all this."*): `js/file-header`, `js/no-default-import`, `js/no-unnecessary-callback-wrapper` (suggest only) in `js`; `ts/no-mergeable-namespace` in `ts`; `js/comment-type` and `js/switch-final-break` in the taste preset.
- **Where it is built:** W-139, with round 1 — every `port` row in the CSVs except the `tailwind` pack's 8, which W-145 builds with the pack, plus `tslint-recheck.csv`'s 6: **2,572 rules**. The two npm names are [W-147](../open/W-142-framework-pack-names.md). W-144 closed.

## 12 · Oxlint (researched 2026-10-30)

Asked as *"auxlint"* — no tool by that name; read as **Oxlint** (Oxc, MIT). Oxlint 1.90.0's `--rules --format=json` lists **872** rules (`method/oxlint-rules.jsonl`), classed in `oxlint.csv`:

| class | rules | note |
|---|--:|---|
| `same-as` | 805 | 795 by name in a CSV above (535 of them `port`, 138 `covered`, 56 `types`), 10 by behaviour |
| `port` | 18 | Oxlint-only: 11 `oxc` (bug checks, from DeepScan and Clippy), 4 `react` (react-refresh, @eslint-react), 3 `react_perf` |
| `out-of-scope` | 46 | `vue` — W-143 |
| `skip` | 3 | `no-async-await`, `no-optional-chaining`, `no-rest-spread-properties`: old-target restrictions |

- **Its categories as a signal:** 38 of our tranche B/C ports are Oxlint `correctness` (outright bugs) — W-148 corpus-runs them for `recommended`.
- **Its formatters (oxfmt 0.75.0) on W-141's corpus:** CSS 88.6% (codelaw 41.8%), Markdown 97.3% (73.3%), YAML 97.3% (88.7%), SCSS 88.0% (82.7%), JS 96.0% (94.7%), TS 94.0% (94.0%). Native Rust for these; HTML, Vue, Svelte and Astro go to Prettier itself. → W-148.
- **Already in codelaw:** the `agent` format and agent detection (SR-CLI-MCP), `--timings`, unused-suppression reports, nested configs, the Oxlint adapter and its type-aware mode, the rival bench (L8). **Not taken:** tsgolint (SR-TYPES R1), JS plugins (packs hold data, not code).
- **Ruled 2026-10-30 (Arpit, voice):** `migrate oxlint|oxfmt` to the backlog (B-066); the rest one work item, W-148 (*"File it."*).
