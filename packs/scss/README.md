# @codelaw/rules-scss · codelaw-rules-scss

codelaw's rules for SCSS (Sass's `.scss` syntax) — one of the first-party rule
packs ([SR-RULE-PACKS](../../records/0101_rule-packs.md)). The same files ship on
npm as `@codelaw/rules-scss` and on PyPI as `codelaw-rules-scss`.

SCSS is a CSS dialect under [L3](../../records/0005_LAW-3-languages.md) (amended
2026-10-26): `codelaw format` and `codelaw check` read `.scss` files with their
own grammar, and **the `css/*` rules of `@codelaw/rules-css` also run on SCSS**
where they declare `scss` — install both packs in an SCSS project. The indented
`.sass` syntax and Less are out of scope.

```toml
# codelaw.toml
extends = ["codelaw:css/recommended", "codelaw:scss/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `scss/recommended` | 7 | correctness: at-rule-no-unknown, duplicate definitions, `@extend` placeholders, null tests, empty comments, partial names |
| `scss/strict` | 15 | the additions to `scss/recommended` (it extends it): Dart Sass's module migrations (no `@import`, no global function names, no slash division), unknown functions, naming patterns and a nesting limit |
| `scss/markers` | 1 | TODO comments with a ticket (and a date when asked); opt in (SR-CONFIG decision 11a) |

<!-- /docs-sync:presets -->

`scss/no-import` is in `strict`, not `recommended`: a codebase that still uses `@import` moves to
`@use` on its own schedule.

## Rules

<!-- docs-sync:rules -->

| rule | fix | flags |
|---|---|---|
| `scss/at-extend-no-missing-placeholder` | manual | `@extend .class` instead of a `%placeholder` |
| `scss/at-if-no-null` | auto | `@if $x != null` (fixed to `$x`) and `@if $x == null` (fixed to `not $x`) |
| `scss/at-mixin-pattern` | manual | mixin names outside the pattern (kebab-case) |
| `scss/at-rule-no-unknown` | manual | at-rules neither CSS nor Sass defines |
| `scss/dollar-variable-pattern` | manual | `$variable` names outside the pattern (kebab-case) |
| `scss/double-slash-comment-empty` | auto | an empty `//` comment |
| `scss/function-no-unknown` | manual | functions that are not CSS, Sass built-ins or the file's own |
| `scss/load-no-partial-leading-underscore` | auto | `@use "_partial"` |
| `scss/max-nesting-depth` | manual | style rules nested deeper than `max` (3) |
| `scss/no-duplicate-dollar-variables` | manual | a `$variable` declared twice in one scope |
| `scss/no-duplicate-mixins` | manual | a mixin defined twice in one scope |
| `scss/no-global-function-names` | auto | `map-get()` and the other built-ins that moved into modules |
| `scss/no-import` | suggest | `@import` of a Sass file |
| `scss/no-slash-division` | suggest | `/` as division |
| `scss/percent-placeholder-pattern` | manual | `%placeholder` names outside the pattern (kebab-case) |

<!-- /docs-sync:rules -->

`$a+$b` is not a rule here: spacing around an operator is layout, and layout is the formatter's ([SR-FORMATTER](../../records/0105_formatter.md) decision 3) — `codelaw format` spaces it.
