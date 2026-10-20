---
type: Handoff
name: W-03
description: "Arpit, 2026-10-19: research the ESLint, typescript-eslint, TSLint, Ruff, Prettier, stylelint and HTML-linter presets and build the five codelaw rule packs from them."
item: W-03
filed: 2026-10-19
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Claude Code, Opus 5.5) — step 6 packaging"
ball: agent
---

# W-03 — Build the five rule packs from the best-known presets

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Research the presets of ESLint, typescript-eslint, TSLint, Ruff, Prettier, stylelint, html-validate, HTMLHint, markdownlint | what each pack ports, and what it must not | ✅ 2026-10-19 (below) |
| 2 | Write `rules-js` `recommended` presets | JS/TS/JSX/TSX/Node/React/Stencil coverage | ✅ 2026-10-20 — `js/recommended` 64 rules · `ts/recommended` ✅ 24 · `react` ✅ 19 · `node` ✅ 3 · `stencil` ✅ 15 · `js/conventions` ✅ 7 |
| 3 | Write `rules-python` `recommended` | Python coverage | ✅ 2026-10-20 — 98 rules (`py/recommended` 96 · `py/conventions` 2) |
| 4 | Write `rules-css` and `rules-html` `recommended` | stylesheet and markup coverage | ✅ 2026-10-20 — `rules-css` 70 rules (`css/recommended` 41 · `css/standard` 28 · `css/conventions` 1) · `rules-html` ✅ 71 rules (`html/recommended` 63 · `html/a11y` 26 · `html/document` 5 · `html/conventions` 3) |
| 5 | Write `rules-data` | package.json, tsconfig, pyproject, CI YAML, Markdown | ✅ 2026-10-20 — 30 rules in 7 presets; schema validation waits on B-030 |
| 6 | Publish all five to npm and PyPI with the engine | `codelaw init` can install them | ⏳ — packaging ✅ 2026-10-20 (`scripts/build-packs.py`: ten artefacts, reproducible, byte parity, installed by real `npm` and `pip`); licence ✅ MIT 2026-10-20 (`LICENSE` at the root and in every pack; `--release` builds); 🔴 the go to publish; the engine's packages (B-038) not built |

**Arpit, 2026-10-19:** *"research the rules preset in ruff, eslint, tslint, prettier, and other and create as separate packages, like 1 for js/ts/node/jsx/tsx, 1 for html, 1 for css, 1 for python and engine will be separate"*

**Model:** Sonnet: every ported rule has a written definition of done (its `ported_from` source's behaviour, Do/Don't per language, `codelaw test` green), so a check catches a bad port.

## Goal

Five packs whose `recommended` presets cover the correctness core of each ecosystem's best-known preset, reimplemented in codelaw's format, each rule tested, none layout-only.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · porting rules | Every ported rule is **reimplemented**, never vendored; its docs are written fresh; `ported_from:` names the source rule(s). | ✅ 2026-10-20 |
| 2 · no layout | Rules the formatter owns are dropped (L7, SR-FORMATTER decision 3); the formatter's `[format]` defaults follow Prettier's (below). | ✅ 2026-10-20 |
| 3 · no types | Rules needing type information or the React Compiler are excluded and listed in each pack's README as "not ported, and why". | ✅ 2026-10-20 |
| 4 · L4 | Every rule has a fix class; `auto` wherever safe. | ✅ 2026-10-20 |
| 5 · tests | `codelaw test` green for every rule; the pack README states its rule count per preset and the ported/not-ported table. | ✅ 2026-10-20 |
| 7 · starter catalogue | The 34 v0 starter rules ([SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 8) placed: in `recommended` with `ported_from:` where they match a source rule, otherwise in the pack's `conventions` preset. | ✅ 2026-10-20 |
| 6 · publish | Ten packages (5 × npm + PyPI) released with the engine, same version (W-01 DoD 2). | ⏳ built, not published |

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
| Data the rules need | — | Property, at-rule, selector and keyword tables shipped inside the pack, from MIT sources only (stylelint's reference tables, `known-css-properties`, `html-tags`); CC0 data such as MDN's is not OSI-approved (L2), so it is not used. |

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
| sort-package-json conventions | `data/package-json` preset (`auto` fix). |

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

## Progress

- **2026-10-19 — `js/recommended`:** all 64 rules of `@eslint/js` 10.0.1's `recommended`, in `packs/js/`; `codelaw test packs/js` green (796 checks). 26 need native rules, added to `codelaw-native`; the rest are `match:` rules. The engine gained five predicates (`write`, `declaration`, `redeclared`, `binding`, `builtin`), data tables for native rules, kinds that match nothing in grammars lacking them, and refusal of duplicate YAML keys.
- **Corpus check:** run over npm's own source and dependencies (1,122 JS files). It found eight false-positive patterns in the first ports, each fixed with a test, and real defects in the corpus: a `clearTimeout` of a never-assigned timer in `@npmcli/agent`, and an operator-precedence bug in `node-gyp` (`a <= b ?? -1`).
- **2026-10-19 — `ts/recommended`:** all 20 rules of typescript-eslint 8.71.2's `recommended` (not type-checked) plus the four core rules its `eslint-recommended` switches on; 88 rules in `packs/js`, 1,048 checks green. A TypeScript corpus (zod, ajv, TanStack Query sources) found two false-positive patterns, fixed with tests.
- **2026-10-19 — `react/recommended`:** 17 rules of eslint-plugin-react 7.37.5 (`recommended` + `jsx-runtime`) and `rules-of-hooks` and `exhaustive-deps` from eslint-plugin-react-hooks 7.1.1; 107 rules in `packs/js`. react-query's sources come out clean, and planted violations are all found.
- **2026-10-20 — `node`, `stencil`, `js/conventions`:** 132 rules in `packs/js`, 1,505 checks green; the Node rules checked against npm's source. The python, css, html and data packs are being built by four agents in parallel worktrees.
- **2026-10-20 — `rules-data`:** 30 rules in 7 presets (`data/recommended` 6, `md/recommended` 16, `data/package-json` 2, `data/tsconfig` 1, `data/pyproject` 2, `data/github-actions` 1, `data/conventions` 2), 216 checks green; natives in `data_{json,yaml,toml,md}.rs`. Not ported: `md/relative-links` (a native rule sees no file path or file system) and SchemaStore validation (B-030). Run over this repo's own records and docs, `md/single-h1` counts the frontmatter `title:` as the H1, as markdownlint does.
- **2026-10-20 — `rules-css`:** 70 native rules in `css_*.rs` over a hand-written CSS reader (tree-sitter-css errors on range media queries, `layer()` imports and percentage keyframe selectors): all 41 of stylelint-config-recommended 18.0.0 and the 28 non-layout rules stylelint-config-standard 40.0.0 adds; 385 checks green. Three rules check less than stylelint, which uses CSSTree's MDN-derived (CC0) syntax data. A 317-file `node_modules` corpus found three false-positive patterns, fixed; a 60-file spot check found none.
- **2026-10-20 — `rules-html`:** 71 native rules from html-validate 11.16.4's `recommended`, `a11y` and `document` presets (layout and cosmetic rules dropped), over an element tree rebuilt with html-validate's implied-tag rules because tree-sitter-html mis-nests `<option>` and `<head>`/`<body>`; element, entity and input-attribute tables from html-validate (MIT); 349 checks green. A 230-file corpus found six false-positive patterns (one a panic on multi-byte text), all fixed; a 51-file spot check over rustdoc and Homebrew docs found none. Not ported: `no-utf8-bom` (the engine strips the BOM before rules run).
- **2026-10-20 — `rules-python`:** 98 rules — 55 of Ruff 0.17.1's 59 `E4`/`E7`/`E9`/`F` rules, all 39 stable flake8-bugbear rules, I001 and RUF022 in `py/recommended`; `py/no-print` and `py/no-tunable-literals` in `py/conventions`; 605 checks green. The first cut was held back: compared with Ruff on pip's source it misread annotation subscripts (`List[int]`) as PEP 695 declarations in the semantic model. Fixed at the source, then reconciled with Ruff on three corpora (CPython 3.9 stdlib, pip 21.2.4, rich 15.0.0): every remaining difference is a `# noqa` line, a blank line the formatter owns, or a documented gap of resolving names without statement order.
- **Steps 2–5 done; step 6 (publish) is what remains:** it needs the packaging (B-038, B-012) and then the decider's ruling to publish (the licence: MIT, ruled 2026-10-20).
- **2026-10-20 — step 6, packaging:** `scripts/build-packs.py` builds `@codelaw/rules-<name>` (npm tarball) and `codelaw-rules-<name>` (`py3-none-any` wheel) for all five packs into `dist/packs/`, with the version from `Cargo.toml`. Byte-reproducible; each artefact checked against `packs/<name>/` file by file. `--verify` installs all five both ways over `fixtures/l3` with every `recommended` preset: 20 diagnostics, identical output. Installed with real `npm i` (Node 24) and `pip install` (Python 3.14); the engine finds them in `node_modules/` and through `VIRTUAL_ENV`; `pip uninstall` leaves nothing. `--release` refuses: there is no `LICENSE`.
- **Every rule is `preview`** (SR-RULE-AUTHORING decision 6), so presets run only with `preview = true` until the corpus job (B-024) promotes them.

## Blockers

None for steps 2–5: W-02 landed the engine, the semantic model, `native:` rules and pack loading on 2026-10-19.

- **Step 6 (publish)** — the packs are built two ways: `scripts/build-packs.py` (local, 2026-10-20) and the release pipeline's `scripts/package.py` (W-04, 2026-10-20). The licence is ruled MIT (SR-PACKAGES decision 10). What remains is the decider's: the one-time registry setup (`RELEASING.md` §One time) and the ruling to publish — the OPEN-WORK inbox.
- **Engine features some ports need, not built yet:** `target:` + `assert:` rules and the `sort-keys`/`ensure-key` primitives (B-030) for `rules-data`'s schema and sorting rules; the project graph (B-029) for `graph/no-unused-exports`. Rules that need them wait in the pack README's "not ported yet" list; they are not approximated.

## Record affected

SR-RULE-PACKS (the packs' contents and presets) · SR-FORMATTER decision 5 (Prettier-derived defaults) · `work/IMPLEMENTATION.md` per pack on landing.

## Hazards

- **Test every rule with `codelaw test packs/<name>`** before it lands; a rule that needs a predicate or primitive the engine lacks is a W-02 follow-up, never a looser rule.

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
- [W-01](../../archive/open/W-01-rule-pack-records.md) · [W-02](W-02-engine-p0.md)
