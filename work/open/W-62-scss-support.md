---
type: Handoff
name: W-62
description: "Arpit, 2026-10-26: SCSS (.scss) joins codelaw as a CSS dialect: L3 amended on the decider's ruling, Biome's SCSS formatter for L7, every css/* rule checked against SCSS, and SCSS-specific rules ported from stylelint-scss (MIT), including Dart Sass's module-system migrations."
item: W-62
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5) — docket"
ball: agent
---

# W-62 — SCSS in the CSS pack — L3 amended, parsed, formatted, linted

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend L3: `scss` joins `languages` as a CSS dialect (like `jsx`/`tsx`); `.sass` indented syntax and Less stay out. On the decider's ruling of 2026-10-26 | the law allows it | ⏳ |
| 2 | **Parser — B-047, ruled 2026-10-26: own grammar.** Extend codelaw's tree-sitter-css fork for the CSS forms that still give `ERROR`, and take **tree-sitter-scss** (MIT, tree-sitter-grammars) for `.scss`; move the CSS rules off `css_syntax.rs` onto these trees. Biome stays the formatter only | CSS and SCSS rules on real trees, under codelaw's control | ⏳ |
| 3 | Formatter (L7): Biome's SCSS formatter at the pinned revision; `format-parity.py` against Prettier's SCSS on a corpus | SCSS formats like CSS does | ⏳ |
| 4 | Every `css/*` rule declares `languages: [css, scss]` or says why not; cases for `$var`, nesting, `@mixin`/`@include`, `#{}` interpolation, `//` comments, placeholders | no false positives on SCSS | ⏳ |
| 5 | SCSS rules (below), ported from stylelint-scss with `ported_from:` | SCSS-specific checks | ⏳ |
| 6 | **A separate pack `scss`** — `@codelaw/rules-scss` on npm, `codelaw-rules-scss` on PyPI (Django/Flask apps use SCSS too) — holding the `scss/*` rules below, presets `scss/recommended` and `scss/strict`; amend L6 (fifteen → seventeen packs, with W-63) and `FIRST_PARTY_PACKS` | SCSS rules installed only by SCSS projects | ⏳ |
| 7 | `codelaw init` detects `.scss` files or `sass` in `package.json` and adds the `scss` pack | adoption | ⏳ |

**Arpit, 2026-10-26:** *"I want max lines rule for HTML CSS files as well. Research and create a work item for it. The CSS pack should work with SCSS, and CSS modules as well."*

**Arpit, 2026-10-26:** *"Create separate packages for SCSS and CSS modules."* So the dialect itself (parser, formatter, `css/*` rules staying quiet on SCSS) lands in the engine and the `css` pack; the `scss/*` rules ship in their own pack.

**Model:** Opus — it amends a law and moves every CSS rule onto new trees.

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
| 1 · | L3 and L7 records amended; register updated | ⏳ |
| 2 · | a `.scss` corpus: no `css/*` rule reports SCSS syntax as an error | ⏳ |
| 3 · | `codelaw format` on SCSS is idempotent and parity-checked | ⏳ |
| 4 · | every SCSS rule has cases; `scss/no-global-function-names` fix adds the `@use` once | ⏳ |


## Rulings

**Arpit, 2026-10-26 (docket):** B-047 — *"Own grammar / tree-sitter-scss"* (over the recommended Biome tree). This item absorbs B-047.

## Blockers

None — B-047 ruled 2026-10-26 (own grammar + tree-sitter-scss).

## Record affected

L3 (SCSS as a CSS dialect) · L7 (its formatter) · SR-LINTER / SR-FORMATTER (the dialect) · L6 (the `scss` pack) · SR-RULE-PACKS (the `scss` pack's row; the css pack's rules declare `scss`) · B-047 closes.

## Hazards

- Two grammars to keep: the tree-sitter-css fork (B-046's upstream PR) and tree-sitter-scss — pin both, test both on the corpus.
- Biome's SCSS formatter is new: pin the revision and gate formatting on the corpus.
- Dart Sass module system vs legacy `@import` codebases: `scss/no-import` belongs in `strict` until a team opts in.

## Reference

[Biome language support](https://biomejs.dev/internals/language-support/) · [stylelint-scss](https://github.com/stylelint-scss/stylelint-scss) · [Sass: @import is deprecated](https://sass-lang.com/documentation/at-rules/import/) · [tree-sitter-scss](https://github.com/tree-sitter-grammars/tree-sitter-scss)
