---
type: Handoff
name: W-03
description: "Arpit, 2026-10-19: research the ESLint, typescript-eslint, TSLint, Ruff, Prettier, stylelint and HTML-linter presets and build the five codelaw rule packs from them."
item: W-03
filed: 2026-10-19
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-19
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-03 — Build the five rule packs from the best-known presets

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Research the presets of ESLint, typescript-eslint, TSLint, Ruff, Prettier, stylelint, html-validate, HTMLHint, markdownlint | what each pack ports, and what it must not | ✅ 2026-10-19 (below) |
| 2 | Write `rules-js` `recommended` presets | JS/TS/JSX/TSX/Node/React/Stencil coverage | ⏳ waiting on W-02 |
| 3 | Write `rules-python` `recommended` | Python coverage | ⏳ |
| 4 | Write `rules-css` and `rules-html` `recommended` | stylesheet and markup coverage | ⏳ |
| 5 | Write `rules-data` | package.json, tsconfig, pyproject, CI YAML, Markdown | ⏳ |
| 6 | Publish all five to npm and PyPI with the engine | `codelaw init` can install them | ⏳ |

**Arpit, 2026-10-19:** *"research the rules preset in ruff, eslint, tslint, prettier, and other and create as separate packages, like 1 for js/ts/node/jsx/tsx, 1 for html, 1 for css, 1 for python and engine will be separate"*

**Model:** Sonnet: every ported rule has a written definition of done (its `ported_from` source's behaviour, Do/Don't per language, `codelaw test` green), so a check catches a bad port.

## Goal

Five packs whose `recommended` presets cover the correctness core of each ecosystem's best-known preset, reimplemented in codelaw's format, each rule tested, none layout-only.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · porting rules | Every ported rule is **reimplemented**, never vendored; its docs are written fresh; `ported_from:` names the source rule(s). | ⏳ |
| 2 · no layout | Rules the formatter owns are dropped (L7, SR-FORMATTER decision 3); the formatter's `[format]` defaults follow Prettier's (below). | ⏳ |
| 3 · no types | Rules needing type information or the React Compiler are excluded and listed in each pack's README as "not ported, and why". | ⏳ |
| 4 · L4 | Every rule has a fix class; `auto` wherever safe. | ⏳ |
| 5 · tests | `codelaw test` green for every rule; the pack README states its rule count per preset and the ported/not-ported table. | ⏳ |
| 6 · publish | Ten packages (5 × npm + PyPI) released with the engine, same version (W-01 DoD 2). | ⏳ |

## Research: what each pack ports (2026-10-19)

**Legend.** *Port* = reimplement in v1 `recommended`. *Later* = `strict` or a later version. *Never* = out of scope by law or record.

### `rules-js` — JS, JSX, TS, TSX, Node, React, Stencil

| Source preset | Size | Plan |
|---|---|---|
| ESLint `js.configs.recommended` (`@eslint/js`) | 64 rules | **Port** all 64 into `js/recommended` (e.g. `no-dupe-keys`, `no-unreachable`, `no-unused-vars`, `no-undef`, `valid-typeof`, `use-isnan`, `preserve-caught-error`). ~20 need the semantic model (W-02 DoD 3). |
| typescript-eslint `recommended`, `strict`, `stylistic` | — | **Port** the non-type-checked `recommended` into `ts/recommended`; `strict` → `js/strict` *later*; `stylistic` only where it isn't layout. **Never:** every `*-type-checked` config (no types; L3). |
| TSLint | — | **Nothing to port.** Deprecated since 2019; typescript-eslint is its replacement and is covered above. |
| eslint-plugin-react `recommended` + `jsx-runtime` | — | **Port** into `react/recommended` (keys in lists, no direct state mutation, etc.). |
| eslint-plugin-react-hooks `recommended` | 2 core + 14 React Compiler rules | **Port** `rules-of-hooks` and `exhaustive-deps`. **Never** (for now): the 14 compiler rules, which need the React Compiler's analysis. |
| eslint-plugin-n `recommended` | — | **Port** into `node/recommended` (deprecated APIs, `process.exit` in libraries, missing imports). |
| @stencil/eslint-plugin `recommended` | — | **Port** into `stencil/recommended`. |
| eslint-plugin-jsx-a11y, unicorn | — | *Later* (`js/strict`, `react/a11y`). |
| oxlint (871 rules, 111 on by default) and Biome `recommended` | — | **Cross-check only:** both are Rust reimplementations of the same presets and show which rules are implementable without types. |

### `rules-python` — Python

| Source preset | Size | Plan |
|---|---|---|
| Ruff's long-standing default `E4, E7, E9, F` (Pyflakes + pycodestyle errors) | 59 rules | **Port** all into `py/recommended` (e.g. `F401` unused import, `F841` unused variable, `F821` undefined name, `E711` comparison to None). |
| Ruff v0.16.0 default (July 2026) | 413 rules, 34 categories (B, UP, SIM, PL, C4, DTZ, I, ASYNC, FURB, RUF, TRY, YTT, PYI, PT, FA, …) | **Port** `B` (bugbear) and `I` (import sorting, an `auto` fix) into `recommended`; the rest → `py/strict` *later*. `UP` rules read `requires-python` from `pyproject.toml`. |
| Pylint | — | **Ideas only, via Ruff's MIT `PL*` ports.** Pylint is GPL-2.0, so no code or docs are copied. |
| pydocstyle `D` | — | *Later*, opt-in preset. |
| Black / Ruff format | — | Not rules: SR-FORMATTER owns them. |

### `rules-css` — CSS

| Source preset | Size | Plan |
|---|---|---|
| stylelint-config-recommended | 41 rules | **Port** all into `css/recommended` (`block-no-empty`, `no-duplicate-selectors`, `property-no-unknown`, `declaration-block-no-duplicate-properties`, `font-family-no-missing-generic-family-keyword`, …). |
| stylelint-config-standard | recommended + modern-convention rules | **Port** the non-layout ones into `css/standard` (modern color and media-range notation); drop anything the formatter owns. |
| Biome CSS rules | 15 stable, ported from stylelint | Cross-check. |
| Data the rules need | — | Property, at-rule and pseudo-class tables from `@mdn/browser-compat-data` (CC0-1.0), shipped inside the pack. |

### `rules-html` — HTML

| Source preset | Plan |
|---|---|
| html-validate `recommended` (superset), `standard` (WHATWG), `a11y` (WCAG), `document` (full-document checks) | **Port** into `html/recommended`, `html/a11y`, `html/document`. Rules in html-validate's `prettier` and `browser` presets are layout/cosmetic: **never**. |
| HTMLHint (tags, attributes, id/class, inline script/style) | Cross-check; port the non-duplicates (`id-unique`, `alt-require`, `inline-script-disabled` → opt-in). |
| markuplint `recommended` | Cross-check for a11y and conformance. |
| Element/attribute data | From html-validate's element metadata (MIT) or the WHATWG spec; shipped inside the pack. |

### `rules-data` — JSON, YAML, TOML, Markdown

| Source | Plan |
|---|---|
| SchemaStore (Apache-2.0) | **Port** schema validation for `package.json`, `tsconfig.json`, `pyproject.toml`, `.github/workflows/*.yml`, with schemas shipped in the pack (no network, L5). |
| markdownlint (all rules on by default, ~53) | **Port** the non-layout rules into `md/recommended` (heading increment, single H1, fenced-code language, link fragments, empty links, alt text). Layout rules (`MD009`, `MD010`, `MD012`, `MD022`, `MD030`, `MD031`, `MD032`, `MD047`, …) belong to the formatter: **never**. |
| yamllint | **Ideas only, from the YAML 1.2 spec** (duplicate keys, truthy values, anchors). yamllint is GPL-3.0, so no code or docs are copied. |
| taplo (MIT) | Cross-check TOML validity rules. |
| sort-package-json conventions | `package-json` preset (`auto` fix). |

### Prettier → the formatter's defaults, not rules

Prettier has no lint rules. Its option defaults become SR-FORMATTER's `[format]` defaults for JS/TS/JSON/CSS/HTML/YAML/Markdown:

| option | default |
|---|---|
| printWidth | 80 |
| tabWidth / useTabs | 2 / false |
| semi | true |
| singleQuote / jsxSingleQuote | false / false |
| quoteProps | as-needed |
| trailingComma | all |
| bracketSpacing / bracketSameLine | true / false |
| arrowParens | always |
| endOfLine | lf |
| proseWrap | preserve |
| htmlWhitespaceSensitivity | css |
| embeddedLanguageFormatting | auto |

## Blockers

**Waiting on W-02** (the engine, semantic model and pack loading), which waits on W-01.

## Record affected

SR-RULE-PACKS (the packs' contents and presets, after W-01) · SR-FORMATTER decision 5 (Prettier-derived defaults) · `work/IMPLEMENTATION.md` per pack on landing.

## Hazards

- **Licences of the sources:** only ideas are ported. ESLint, typescript-eslint, Ruff, stylelint, html-validate, HTMLHint, markdownlint, Prettier and Biome are MIT; Pylint (GPL-2.0) and yamllint (GPL-3.0) contribute ideas only, never text or code (L2).
- **Rule-for-rule parity is not the goal.** A ported rule matches its source's documented behaviour on the source's own examples; where codelaw differs (e.g. a better fix), the rule's docs say so.
- **Preset drift:** upstream presets change (Ruff's default went from 59 to 413 rules in v0.16.0). Each pack README records the upstream version it was ported against.

## Reference

- [ESLint recommended config](https://cdn.jsdelivr.net/npm/@eslint/js/src/configs/eslint-recommended.js) · [typescript-eslint shared configs](https://typescript-eslint.io/users/configs/) · [What about TSLint?](https://typescript-eslint.io/users/what-about-tslint)
- [eslint-plugin-react-hooks](https://github.com/react/react/blob/main/packages/eslint-plugin-react-hooks/README.md) · [oxlint rules](https://oxc.rs/docs/guide/usage/linter/rules.html)
- [Ruff rules](https://docs.astral.sh/ruff/rules/) · [Ruff 0.16.0 default rules](https://pydevtools.com/blog/ruff-0-16-0-default-rules/) · [Ruff default-rules discussion](https://github.com/astral-sh/ruff/discussions/23203)
- [stylelint-config-recommended](https://cdn.jsdelivr.net/npm/stylelint-config-recommended/index.js) · [stylelint-config-standard](https://fastly.jsdelivr.net/npm/stylelint-config-standard@40.0.0/README.md)
- [html-validate presets](https://html-validate.org/rules/presets.html) · [HTMLHint rules](https://htmlhint.com/docs/user-guide/list-rules)
- [markdownlint](https://github.com/DavidAnson/markdownlint) · [Prettier options](https://prettier.io/docs/options)
- [W-01](W-01-rule-pack-records.md) · [W-02](W-02-engine-p0.md)
