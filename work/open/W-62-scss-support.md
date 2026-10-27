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
| 2 | **Parser — B-047, ruled 2026-10-26: own grammar.** Extend codelaw's tree-sitter-css fork for the CSS forms that still give `ERROR`, and take **tree-sitter-scss** (MIT, tree-sitter-grammars) for `.scss`; move the CSS rules off `css_syntax.rs` onto these trees. Biome stays the formatter only | CSS and SCSS rules on real trees, under codelaw's control | ◐ both grammars patched (2026-10-27): the CSS fork reads the forms that gave `ERROR`, the SCSS fork (built on it) reads Bootstrap's SCSS — 233 of 236 files without an error, from 52; the `css/*` rules are **not** yet moved onto the trees — see *Left* |
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
| 2 · | a `.scss` corpus: no `css/*` rule reports SCSS syntax as an error | ✅ `fixtures/scss/corpus/modern.scss` (valid modern Sass), pinned in `fixtures/scss/css-pack.toml`; `tests/scss.rs`. Bootstrap 6's real SCSS is not in the pinned corpus — see *Left* |
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

## Left (honest)

1. **Step 2's second half: move the `css/*` (and `scss/*`) rules from `css_syntax.rs` onto the trees.** The grammars now read the forms SR-ARCHITECTURE decision 15b named as the reason the rules read text; the move itself — about 70 native rules across `css_atrules.rs`, `css_declarations.rs`, `css_media.rs`, `css_selectors.rs`, `css_values.rs`, `css_misc.rs`, `css_modules.rs` and `scss_rules.rs` — has not started. Rules that exist to report CSS the grammar rejects (`css/media-query-no-invalid`, `css/at-rule-prelude-no-invalid`) keep reading text.
2. **Grammar forms still read as `ERROR`** (the READMEs list them): SCSS `@container #{$name} (…)`, a string holding an interpolation with its own quotes, `:not(.a)#{$x}`, an escape after an interpolation; CSS an unquoted `url(data:…;…)` holding `;`, `@nest`, `@value` (W-63), a backslash-newline in a string. The CSS fork's `upstream/grammar.patch` covers the 2026-10-22 forms only.
3. **Pinned corpora** (`scripts/corpus.json`): an SCSS one (Bootstrap) and a CSS Modules one (W-63's step), a `corpus.toml` count for each `scss/*` and `css-modules/*` rule, then `stable` for those with no false positive. Fetching works on this machine (2026-10-27), so it does not wait on CI.
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
