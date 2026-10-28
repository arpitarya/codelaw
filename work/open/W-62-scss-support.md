---
type: Handoff
name: W-62
description: "Arpit, 2026-10-26: SCSS (.scss) joins codelaw as a CSS dialect: L3 amended on the decider's ruling, Biome's SCSS formatter for L7, every css/* rule checked against SCSS, and SCSS-specific rules ported from stylelint-scss (MIT), including Dart Sass's module-system migrations."
item: W-62
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5)"
ball: agent
---

# W-62 — SCSS in the CSS pack — L3 amended, parsed, formatted, linted

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend L3: `scss` joins `languages` as a CSS dialect (like `jsx`/`tsx`); `.sass` indented syntax and Less stay out. On the decider's ruling of 2026-10-26 | the law allows it | ✅ 2026-10-26 |
| 2 | **Parser — B-047, ruled 2026-10-26: own grammar.** Extend codelaw's tree-sitter-css fork for the CSS forms that still give `ERROR`, and take **tree-sitter-scss** (MIT, tree-sitter-grammars) for `.scss`; move the CSS rules off `css_syntax.rs` onto these trees. Biome stays the formatter only | CSS and SCSS rules on real trees, under codelaw's control | ◐ both grammars patched (2026-10-27); every CSS/SCSS rule reads its **statements and style-rule selectors from the tree** (2026-10-27, `css_tree.rs`); component values, preludes/media queries and pseudo-class arguments still read as text — see *Left* |
| 3 | Formatter (L7): Biome's SCSS formatter at the pinned revision; `format-parity.py` against Prettier's SCSS on a corpus | SCSS formats like CSS does | ✅ Biome's SCSS source; 98.6 % identical files vs Prettier (212 files) |
| 4 | Every `css/*` rule declares `languages: [css, scss]` or says why not; cases for `$var`, nesting, `@mixin`/`@include`, `#{}` interpolation, `//` comments, placeholders | no false positives on SCSS | ✅ 64 of 71 declare `scss`; a two-file SCSS corpus with a pinned count per rule |
| 5 | SCSS rules (below), ported from stylelint-scss with `ported_from:` | SCSS-specific checks | ✅ 15 of 16: `scss/operator-no-unspaced` is a layout rule (SR-FORMATTER decision 3) |
| 6 | **A separate pack `scss`** — `@codelaw/rules-scss` on npm, `codelaw-rules-scss` on PyPI (Django/Flask apps use SCSS too) — holding the `scss/*` rules below, presets `scss/recommended` and `scss/strict`; amend L6 (fifteen → seventeen packs, with W-63) and `FIRST_PARTY_PACKS` | SCSS rules installed only by SCSS projects | ✅ `@codelaw/rules-scss` + `codelaw-rules-scss`; L6 and `FIRST_PARTY_PACKS` sixteen packs (W-63 makes it seventeen) |
| 7 | `codelaw init` detects `.scss` files or `sass` in `package.json` and adds the `scss` pack | adoption | ✅ `.scss` files or `sass`/`sass-embedded`/`node-sass` in `package.json` add `scss` and `css` |

**Arpit, 2026-10-26:** *"I want max lines rule for HTML CSS files as well. Research and create a work item for it. The CSS pack should work with SCSS, and CSS modules as well."*

**Arpit, 2026-10-26:** *"Create separate packages for SCSS and CSS modules."* So the dialect itself (parser, formatter, `css/*` rules staying quiet on SCSS) lands in the engine and the `css` pack; the `scss/*` rules ship in their own pack.

**Model:** Opus — it amends a law and moves every CSS rule onto new trees.

**Arpit, 2026-10-27 (voice docket):** *"Ratify it."* — `tree-sitter-cli` 0.25.10 is **D-36** in L11's list, on the terms put to the decider: exact pin, the regenerated `src/` committed (so CI, the engine and users never need the tool), run only when a grammar changes. `work/BLOCKED.json` back to `PROCEED`; W-62 re-balled to agent. *Left* items 1–2 can start.

## Goal

### SCSS rules (from stylelint-scss, MIT)

| rule | flags | fix |
|---|---|---|
| `scss/no-import` | `@import` — deprecated in Dart Sass, removed in 3.0 | suggest: `@use`/`@forward` |
| `scss/no-global-function-names` | global built-ins (`map-get`, `darken`, `percentage`) | auto: `map.get` + `@use "sass:map"` |
| `scss/no-slash-division` | `/` as division | suggest: `math.div` |
| `scss/at-rule-no-unknown` | unknown at-rules, SCSS-aware (replaces `css/at-rule-no-unknown` on `.scss`) | manual |
| `scss/no-duplicate-dollar-variables` | a `$var` declared twice in one scope | manual |
| `scss/dollar-variable-pattern` | `$var` names (option: pattern, default kebab-case) | manual |
| `scss/at-mixin-pattern` · `scss/percent-placeholder-pattern` | mixin and placeholder names | manual |
| `scss/at-extend-no-missing-placeholder` | `@extend .class` instead of a `%placeholder` | manual |
| `scss/load-no-partial-leading-underscore` | `@use "_partial"` | auto: drop `_` |
| `scss/at-if-no-null` | `@if $x == null` | auto: `@if not $x` |
| `scss/operator-no-unspaced` | `$a+$b` | auto |
| `scss/double-slash-comment-empty` | empty `//` | auto: remove |
| `scss/no-duplicate-mixins` | a mixin defined twice | manual |
| `scss/function-no-unknown` | unknown functions, module-aware | manual |
| `scss/max-nesting-depth` | nesting deeper than `max` (default 3) | manual |

- `.sass` (indented syntax) and Less: out; named in the README.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | L3 and L7 records amended; register updated | ✅ |
| 2 · | a `.scss` corpus: no `css/*` rule reports SCSS syntax as an error | ✅ `fixtures/scss/corpus/modern.scss` (valid modern Sass), pinned in `fixtures/scss/css-pack.toml`; `tests/scss.rs`; since 2026-10-27 Bootstrap 5.3.8's real SCSS is in the pinned corpus (`packs/css/corpus.toml`, `packs/scss/corpus.toml`) |
| 3 · | `codelaw format` on SCSS is idempotent and parity-checked | ✅ `tests/scss.rs`; `scripts/format-parity.py` measures SCSS (SR-FORMATTER consequences) |
| 4 · | every SCSS rule has cases; `scss/no-global-function-names` fix adds the `@use` once | ✅ `packs/scss/rules/scss/*.cases.md` (`two calls add the @use once`) |


## Built (2026-10-26, Claude Code)

- `Language::Scss` (`.scss`, `scss` fence and heading), `Grammar::Scss` over `crates/codelaw-lang/grammars/tree-sitter-scss` (1.0.0, commit `5da5ba7`, unchanged `src/`, own `bindings/rust`; L11's list D-33), the grammar fingerprint, Biome's SCSS formatter and parser check, `format` markers for `//`, the config schema, the VS Code extension, `codelaw init`.
- The CSS statement reader has an SCSS mode (`css_syntax::parse_scss`): `//` comments blanked (same offsets), `#{}` interpolation as one unit, Sass at-rules and nested-property blocks kept as `Other` statements, `&-suffix` as one nesting selector, `!default`/`!global`/`!optional` not unknown annotations, `rgba($c, $a)` not an alias, `&` inside a mixin rooted. The 70 `css_rule!` rules run on `Scss`; 64 declare it. Rules run on the root even when it is `ERROR` (tree-sitter-scss cannot read all modern Sass), once.
- The `scss` pack: 15 rules, `scss/recommended` (7) and `scss/strict` (extends it, +8), data tables `scss/at-rules` and `scss/functions`.

## Built (2026-10-27, Claude Code) — the grammar patches

- **The CSS fork, `0.25.0+codelaw.3`:** `supports()` in `@import`, a name before an at-rule's query (`@container sidebar (…)`, `@custom-media`, `@font-feature-values Font One`), page selectors (`@page :first`), named-range keyframes (`entry 0%`), `.b &`, a custom property holding a block, `!ie`-style annotations; `test/corpus/modern.txt` has one case per form. On 738 stylesheets the files without an error went from 586 to 723 and every tree that had none is byte-identical.
- **The SCSS fork, `1.0.0+codelaw.2`:** `grammar.js` extends the CSS fork (not npm `tree-sitter-css` 0.20, which L11 has not ratified) and reads the module system, maps, flags, `@if not`/`and`/`or`, rest and keyword arguments, `using`, interpolation in property names, placeholders, pseudo-elements, media features and `:nth-child()`, nested properties, signed variables and `if(sass(…): …; else: …)`; its scanner skips `#{…}`. `test/corpus/codelaw.txt` has one case per change (79 corpus tests pass). Bootstrap 5.3.8 + `main` (`d793f28`): 233 of 236 files without an error, from 52; no file that parsed clean gained an error. The folders' READMEs list every change and what is still not read.
- `src/` regenerated with exactly `npx -y tree-sitter-cli@0.25.10 generate` (D-36) and committed; both `+codelaw.N` bumps are in `Cargo.lock` and `GRAMMARS_FINGERPRINT`; `codelaw-lang`'s `modern_css_parses` grows nine forms and `modern_scss_parses` is new.
- **`scss/at-if-no-null` fixes `== null` too** (to `not $x`), now that the verifier's grammar reads `@if not`; two new cases.
- Parse throughput on the same files (tree-sitter CLI, 3 runs): CSS ~20 → ~21 MB/s, SCSS ~8.5 → ~24 MB/s. `codelaw bench` main vs branch, alternated twice on this loaded Mac (load ~21): `check_cold_10k` median 6907/5760 ms vs 6318/5793 ms; warm runs are 10 001 cache hits and parse nothing.

## Built (2026-10-27, Claude Code) — the pinned SCSS corpus

- `scripts/corpus.json`: **Bootstrap 5.3.8** (`25aa8cc`, MIT) for the `css` and `scss` packs, and the `scss` pack on djangoproject.com; `scss` joins `corpus.py`'s `PACKS`. Fetched and run on this machine; `packs/scss/corpus.toml` has a count for every `scss/*` rule (2 repos, 986 files, 798 findings) and `packs/css/corpus.toml` grows with Bootstrap (1 278 files).
- **Five SCSS false positives the corpus showed, fixed, each with a case:** `css/custom-property-pattern` on interpolated names (`--#{$prefix}color`: 1 295 on Bootstrap's `scss/`); words of a `//` comment inside a selector list read as type selectors (`css/selector-type-no-unknown`) — every `css/*` rule now reads SCSS with `//` comments blanked, and drops a fix that would overwrite one; `css/no-duplicate-selector` and `css/no-descending-specificity` treated a Sass block (`@include respond-min(…) { … }`) as the enclosing context; `css/no-invalid-position-declaration` on a declaration under a mixin's `@media`; and, from the blanking, `css/no-empty-block` / `css/no-empty-source` now count a `//` comment as content.
- **Counts that moved on the old corpus, and why:** on djangoproject.com's SCSS `css/no-descending-specificity` 350 → 319, `css/no-duplicate-selector` 26 → 15, `css/selector-type-no-unknown` 1 → 0 — each removed finding checked by hand: rules inside different `@include respond-min/max(…)` blocks, and `//display none list` inside a selector list. The rest of `packs/css/corpus.toml`'s changes are Bootstrap's own findings (most from `dist/css`).

## Built (2026-10-27, Claude Code) — the rules onto the trees: statements and selectors

- **`crates/codelaw-native/src/css_tree.rs`** builds each stylesheet's sheet from its tree (the CSS fork, or the SCSS fork for `.scss`): rules, keyframe blocks, at-rules (Sass's as `Other`, `@else`/`@else if` clauses as statements of their own), declarations (a custom property's block kept in its value, a Sass nested property an `Other` without children), comments; block, colon, `!` annotation and `;` positions from the tree's nodes. Each style rule also carries its **selector list from the tree** (`Sheet::selectors`): compounds, simple selectors (class, id, type with namespace, universal, attribute, pseudo-class/element with argument range, `&` and `&-suffix`), combinators, a leading combinator.
- **Every `css/*`, `css-modules/*` and `scss/*` rule reads that sheet** (`css_rule!`, `module_rule!`, the `scss_rules.rs` macro and `css-modules/composes-valid`'s target file). The selector rules (`css_selectors.rs`, `css_modules.rs`) take rule selectors from `Sheet::selectors`.
- **The text reader stays the fallback, by design:** a tree with an `ERROR`/`MISSING` node (the CSS the grammar rejects: 7 of 198 stylesheets in each pinned corpus, 11 of 460 case inputs), a CSS file holding a `//` comment, and interpolated/templated selectors (`.a-#{$x}`, which the rules skip). The two prelude rules (`css/media-query-no-invalid`, `css/at-rule-prelude-no-invalid`) take the same sheet and judge the prelude text, as SR-ARCHITECTURE 15b says.
- **Checked equal, not assumed:** a throwaway harness compared the tree's sheet with the text's on every stylesheet of `target/corpus/packs/{css,scss}` and every css/scss fence in the packs' rule and case files — 191 + 187 + 449 sheets identical, 12 547 + 12 477 + 1 595 selector lists identical; `css_tree::tests` keeps one sample per form. `codelaw check` with `css/recommended`+`standard`+`conventions`+`scss/strict` over the css corpus's 198 stylesheets: the same 24 640 findings on main and on the branch (positions and rules); both pinned corpus counts unchanged (css 39 785, scss 798); cases css 836, scss 165, css-modules 62 pass.
- **L8:** release builds, same machine, alternated: single-threaded `check --no-cache` on those 198 stylesheets, 11 runs each, median 2.457 s main vs 2.416 s branch; multi-threaded 7 runs, median 0.921 vs 0.888 s. (A first cut was 20 % slower: the two prelude rules asked for a text sheet and thrashed the one-sheet cache; one sheet for all rules, a shared cursor, and kind names looked up by `kind_id` brought it under main.) CI's perf job is the gate.
- **`css/no-descending-specificity` (*Left* item 3, checked):** it already resolves `&` against its parents (`resolved()`); the 43–48 findings on Bootstrap's `scss/` compare the resolved selectors (`.accordion-button:not(.collapsed)::after` then `.accordion-button::after`; `.modal-dialog-scrollable .modal-content` then `.modal-content`) and are what the rule (and stylelint's) reports by definition, not false. What made them read as false was the message, which named the unresolved `&::after`: it now names both resolved selectors (`` `&::after` (`.accordion-button::after`) comes after `.accordion-button:not(.collapsed)::after` (line 51) ``). Counts unchanged.

## Left (honest)

1. **Step 2's rest: the parts of a statement the rules still read as text.** Statements and style-rule selectors come from the tree (above); still read by `css_syntax.rs`'s lexers inside the tree's ranges: **component values** (`values()`: the `css_values.rs` and `css_declarations.rs` rules, `scss/no-slash-division`, `scss/no-global-function-names`, `scss/function-no-unknown`), **at-rule preludes and media queries** (`css_atrules.rs`, `css_media.rs`; `css/media-query-no-invalid` and `css/at-rule-prelude-no-invalid` stay on text by design), **selectors inside a pseudo-class's arguments** (`all_simples`, `each_selector_simple`, specificity) and keyframe selectors. Next family: declaration values from the tree's value nodes, with the same corpus equality check.
2. **Grammar forms still read as `ERROR`** (the READMEs list them; such a file's sheet comes from the text): an attribute selector's case flag (`[type="a" i]`, found 2026-10-27), SCSS `@container #{$name} (…)`, a string holding an interpolation with its own quotes, `:not(.a)#{$x}`, an escape after an interpolation; CSS an unquoted `url(data:…;…)` holding `;`, `@nest`, `@value` (W-63), a backslash-newline in a string. The CSS fork's `upstream/grammar.patch` covers the 2026-10-22 forms only.
3. **After the corpus:** `stable` for the `scss/*` rules waits on W-100's criterion (no rule is promoted until it is ratified); a CSS Modules corpus is W-63's step. (`css/no-descending-specificity` and `&`: done 2026-10-27, see *Built*.)
4. **The graph does not read `.scss`:** `graph/no-unused-css-classes` and `css/project` stay CSS-only (W-63 extends the graph for modules).
5. **Not shipped on purpose:** `scss/operator-no-unspaced` (layout; the formatter's) — the item's table listed it.

## Rulings

**Arpit, 2026-10-26 (docket):** B-047 — *"Own grammar / tree-sitter-scss"* (over the recommended Biome tree). This item absorbs B-047.

## Blockers

None. **Resolved 2026-10-27** (D-36 ratified; the grammars were patched with it). The ask was: **ASK, 2026-10-26 (`work/BLOCKED.json`):** *Left* items 1 and 2 need the tree-sitter CLI (npm `tree-sitter-cli` 0.25.10, MIT, build-time only) to regenerate `src/` from a patched `grammar.js`. It is not installed on this machine and not in L11's list, so L11 forbids it until the decider ratifies it by name. Question: ratify `tree-sitter-cli` 0.25.10? On a yes: add the D-row, patch both grammars, bump to `1.0.0+codelaw.1`, move the CSS rules onto trees, re-ball to agent. Item 3 (corpus) needs CI network and is independent. B-047 itself was ruled 2026-10-26.

## Record affected

L3 (SCSS as a CSS dialect) · L7 (its formatter) · SR-LINTER / SR-FORMATTER (the dialect) · SR-ARCHITECTURE decisions 3 and 15b (both grammar forks; why the CSS rules still read text) · L6 (the `scss` pack) · SR-RULE-PACKS (the `scss` pack's row; the css pack's rules declare `scss`) · B-047 closes.

## Hazards

- Two grammars to keep: the tree-sitter-css fork (B-046's upstream PR) and tree-sitter-scss — pin both, test both on the corpus.
- Biome's SCSS formatter is new: pin the revision and gate formatting on the corpus.
- Dart Sass module system vs legacy `@import` codebases: `scss/no-import` belongs in `strict` until a team opts in.

## Reference

[Biome language support](https://biomejs.dev/internals/language-support/) · [stylelint-scss](https://github.com/stylelint-scss/stylelint-scss) · [Sass: @import is deprecated](https://sass-lang.com/documentation/at-rules/import/) · [tree-sitter-scss](https://github.com/tree-sitter-grammars/tree-sitter-scss)
