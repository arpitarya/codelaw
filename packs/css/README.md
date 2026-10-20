# @codelaw/rules-css · codelaw-rules-css

codelaw's rules for CSS — one of the five first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). The same files ship on npm
as `@codelaw/rules-css` and on PyPI as `codelaw-rules-css`. The rules also run
on the CSS inside HTML: `<style>` elements and `style=""` attributes.

```toml
# codelaw.toml
extends = ["codelaw:css/recommended", "codelaw:css/standard"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

| preset | rules | ported from |
|---|---|---|
| `css/recommended` | 41 | stylelint-config-recommended 18.0.0 (stylelint 17.16.0) — all 41 rules |
| `css/standard` | 28 | the rules stylelint-config-standard 40.0.0 adds to the recommended set, less the 13 layout rules (see below). It holds only the additions: extend it together with `css/recommended` |
| `css/conventions` | 1 | house conventions, opt-in: `css/no-important` |

Three starter rules match stylelint rules and sit in `css/recommended` under
their starter ids: `css/no-empty-block` (`block-no-empty`),
`css/no-duplicate-selector` (`no-duplicate-selectors`) and
`css/no-unknown-property` (`property-no-unknown`). Every other rule keeps its
stylelint name under `css/`.

Where stylelint-config-recommended sets options (`at-rule-prelude-no-invalid`,
`declaration-block-no-duplicate-properties`, `selector-type-no-unknown`,
`string-no-newline`), those are the codelaw rule's defaults. Where
stylelint-config-standard picks a style (`"modern"`, `"percentage"`, `"short"`,
kebab-case patterns), the codelaw rule implements that style; the patterns stay
configurable through each rule's `pattern` option.

Every rule is **reimplemented**, not vendored: a native Rust implementation in
`crates/codelaw-native/src/css_*.rs` reads the stylesheet with its own small CSS
reader, and the rule files, messages and docs are written fresh. Each rule's
`ported_from:` names the source rule.

## How the ports differ

| rule | difference |
|---|---|
| `css/at-rule-descriptor-value-no-unknown` | keyword-valued descriptors and the `@property` `syntax` type only; stylelint uses the full CSSTree grammar |
| `css/at-rule-no-deprecated` | only `@nest` has a fix |
| `css/at-rule-prelude-no-invalid` | `@property`, `@font-palette-values`, `@position-try`, `@counter-style`, `@keyframes`, `@layer`, `@import`, `@namespace`, `@charset` and prelude-less at-rules; `@supports`, `@container`, `@page` and `@scope` preludes are not checked |
| `css/no-empty-block` | `@layer name {}` is not reported: it fixes the order of cascade layers |
| `css/declaration-property-value-no-unknown` | single-keyword values of about forty keyword-valued properties, from a table written for this pack; stylelint validates every value with CSSTree |
| `css/function-calc-no-unspaced-operator` | whitespace other than one space around an operator is left to the formatter |
| `css/no-descending-specificity` | a nested `&` is resolved against each parent selector separately, not as `:is()` of all of them |
| `css/no-unknown-property` | properties from `known-css-properties` plus stylelint's list of previously known names, not CSSTree; a fix is offered when a known name is one or two edits away |
| `css/color-function-notation` | values with `var()` inside the function are skipped |
| `css/declaration-block-no-redundant-longhand-properties` | reported without a fix |
| `css/media-feature-name-no-vendor-prefix` | reported without a fix; the advice is `resolution` with `dppx` |
| `css/media-feature-range-notation` | reported without a fix: tree-sitter-css cannot parse the range syntax, so the engine's syntax guard refuses the rewrite |

Rules whose data stylelint takes from CSSTree (whose syntax data derives from
MDN data, CC0-1.0, which is not an OSI licence) use the tables listed under
[Data](#data) instead.

## Ported

| codelaw rule | stylelint rule | preset | fix |
|---|---|---|---|
| `css/annotation-no-unknown` | `annotation-no-unknown` | recommended | manual |
| `css/at-rule-descriptor-no-unknown` | `at-rule-descriptor-no-unknown` | recommended | manual |
| `css/at-rule-descriptor-value-no-unknown` | `at-rule-descriptor-value-no-unknown` | recommended | manual |
| `css/at-rule-no-deprecated` | `at-rule-no-deprecated` | recommended | auto |
| `css/at-rule-no-unknown` | `at-rule-no-unknown` | recommended | manual |
| `css/at-rule-prelude-no-invalid` | `at-rule-prelude-no-invalid` | recommended | manual |
| `css/no-empty-block` | `block-no-empty` | recommended | auto |
| `css/comment-no-empty` | `comment-no-empty` | recommended | auto |
| `css/custom-property-no-missing-var-function` | `custom-property-no-missing-var-function` | recommended | suggest |
| `css/declaration-block-no-duplicate-custom-properties` | `declaration-block-no-duplicate-custom-properties` | recommended | manual |
| `css/declaration-block-no-duplicate-properties` | `declaration-block-no-duplicate-properties` | recommended | suggest |
| `css/declaration-block-no-shorthand-property-overrides` | `declaration-block-no-shorthand-property-overrides` | recommended | manual |
| `css/declaration-property-value-keyword-no-deprecated` | `declaration-property-value-keyword-no-deprecated` | recommended | suggest |
| `css/declaration-property-value-no-unknown` | `declaration-property-value-no-unknown` | recommended | manual |
| `css/font-family-no-duplicate-names` | `font-family-no-duplicate-names` | recommended | auto |
| `css/font-family-no-missing-generic-family-keyword` | `font-family-no-missing-generic-family-keyword` | recommended | manual |
| `css/function-calc-no-unspaced-operator` | `function-calc-no-unspaced-operator` | recommended | suggest |
| `css/keyframe-block-no-duplicate-selectors` | `keyframe-block-no-duplicate-selectors` | recommended | manual |
| `css/keyframe-declaration-no-important` | `keyframe-declaration-no-important` | recommended | suggest |
| `css/media-feature-name-no-unknown` | `media-feature-name-no-unknown` | recommended | manual |
| `css/media-feature-name-value-no-unknown` | `media-feature-name-value-no-unknown` | recommended | manual |
| `css/media-query-no-invalid` | `media-query-no-invalid` | recommended | manual |
| `css/media-type-no-deprecated` | `media-type-no-deprecated` | recommended | manual |
| `css/named-grid-areas-no-invalid` | `named-grid-areas-no-invalid` | recommended | manual |
| `css/nesting-selector-no-missing-scoping-root` | `nesting-selector-no-missing-scoping-root` | recommended | manual |
| `css/no-descending-specificity` | `no-descending-specificity` | recommended | manual |
| `css/no-duplicate-at-import-rules` | `no-duplicate-at-import-rules` | recommended | suggest |
| `css/no-duplicate-selector` | `no-duplicate-selectors` | recommended | manual |
| `css/no-empty-source` | `no-empty-source` | recommended | manual |
| `css/no-invalid-double-slash-comments` | `no-invalid-double-slash-comments` | recommended | manual |
| `css/no-invalid-position-at-import-rule` | `no-invalid-position-at-import-rule` | recommended | manual |
| `css/no-invalid-position-declaration` | `no-invalid-position-declaration` | recommended | manual |
| `css/no-irregular-whitespace` | `no-irregular-whitespace` | recommended | suggest |
| `css/property-no-deprecated` | `property-no-deprecated` | recommended | suggest |
| `css/no-unknown-property` | `property-no-unknown` | recommended | suggest |
| `css/selector-anb-no-unmatchable` | `selector-anb-no-unmatchable` | recommended | manual |
| `css/selector-pseudo-class-no-unknown` | `selector-pseudo-class-no-unknown` | recommended | manual |
| `css/selector-pseudo-element-no-unknown` | `selector-pseudo-element-no-unknown` | recommended | manual |
| `css/selector-type-no-unknown` | `selector-type-no-unknown` | recommended | manual |
| `css/string-no-newline` | `string-no-newline` | recommended | manual |
| `css/syntax-string-no-invalid` | `syntax-string-no-invalid` | recommended | manual |
| `css/alpha-value-notation` | `alpha-value-notation` | standard | suggest |
| `css/at-rule-no-vendor-prefix` | `at-rule-no-vendor-prefix` | standard | suggest |
| `css/block-no-redundant-nested-style-rules` | `block-no-redundant-nested-style-rules` | standard | suggest |
| `css/color-function-alias-notation` | `color-function-alias-notation` | standard | suggest |
| `css/color-function-notation` | `color-function-notation` | standard | suggest |
| `css/color-hex-length` | `color-hex-length` | standard | auto |
| `css/container-name-pattern` | `container-name-pattern` | standard | manual |
| `css/custom-media-pattern` | `custom-media-pattern` | standard | manual |
| `css/custom-property-pattern` | `custom-property-pattern` | standard | manual |
| `css/declaration-block-no-redundant-longhand-properties` | `declaration-block-no-redundant-longhand-properties` | standard | manual |
| `css/hue-degree-notation` | `hue-degree-notation` | standard | auto |
| `css/import-notation` | `import-notation` | standard | auto |
| `css/keyframe-selector-notation` | `keyframe-selector-notation` | standard | auto |
| `css/keyframes-name-pattern` | `keyframes-name-pattern` | standard | manual |
| `css/layer-name-pattern` | `layer-name-pattern` | standard | manual |
| `css/length-zero-no-unit` | `length-zero-no-unit` | standard | auto |
| `css/lightness-notation` | `lightness-notation` | standard | auto |
| `css/media-feature-name-no-vendor-prefix` | `media-feature-name-no-vendor-prefix` | standard | manual |
| `css/media-feature-range-notation` | `media-feature-range-notation` | standard | manual |
| `css/number-max-precision` | `number-max-precision` | standard | suggest |
| `css/property-no-vendor-prefix` | `property-no-vendor-prefix` | standard | suggest |
| `css/selector-class-pattern` | `selector-class-pattern` | standard | manual |
| `css/selector-id-pattern` | `selector-id-pattern` | standard | manual |
| `css/selector-no-vendor-prefix` | `selector-no-vendor-prefix` | standard | suggest |
| `css/selector-not-notation` | `selector-not-notation` | standard | suggest |
| `css/selector-pseudo-element-colon-notation` | `selector-pseudo-element-colon-notation` | standard | auto |
| `css/shorthand-property-no-redundant-values` | `shorthand-property-no-redundant-values` | standard | auto |
| `css/value-no-vendor-prefix` | `value-no-vendor-prefix` | standard | suggest |
| `css/no-important` | — | conventions | manual |

## Not ported, and why

Every rule of stylelint-config-recommended is ported. These are the rules
stylelint-config-standard adds that are layout, which the formatter owns
([SR-FORMATTER](../../records/0105_formatter.md)):

| rule | why |
|---|---|
| `at-rule-empty-line-before` | never: layout (blank lines) |
| `comment-empty-line-before` | never: layout (blank lines) |
| `comment-whitespace-inside` | never: layout (whitespace) |
| `custom-property-empty-line-before` | never: layout (blank lines) |
| `declaration-block-single-line-max-declarations` | never: layout (line breaks) |
| `declaration-empty-line-before` | never: layout (blank lines) |
| `font-family-name-quotes` | never: layout (quotes) |
| `function-name-case` | never: layout (case) |
| `function-url-quotes` | never: layout (quotes) |
| `rule-empty-line-before` | never: layout (blank lines) |
| `selector-attribute-quotes` | never: layout (quotes) |
| `selector-type-case` | never: layout (case) |
| `value-keyword-case` | never: layout (case) |

## Upstream versions

- stylelint 17.16.0 — rule behaviour and documentation, read to reimplement.
- stylelint-config-recommended 18.0.0 and stylelint-config-standard 40.0.0 — the rule sets.

## Data

`data/css/*.json` are the tables the rules read. See `NOTICE` for sources and licences.

| table | holds | source |
|---|---|---|
| `css/properties` | known property names | known-css-properties 0.37.0, plus stylelint's `previouslyKnownProperties` |
| `css/at-rules` | known, deprecated and nesting at-rules, page-margin boxes | stylelint `reference/atKeywords.mjs` |
| `css/selectors` | pseudo-classes and pseudo-elements | stylelint `reference/selectors.mjs` |
| `css/type-selectors` | HTML, SVG and MathML element names | stylelint `reference/selectors.mjs` with html-tags 5.1.0; svg-tags 1.0.0; mathml-tag-names 4.0.0 |
| `css/media` | media features, their values and deprecated media types | stylelint `reference/mediaFeatures.mjs`, `reference/mediaTypes.mjs` |
| `css/units` | length units and all units | stylelint `reference/units.mjs` |
| `css/keywords` | CSS-wide, font-family, system-font and colour keywords | stylelint `reference/keywords.mjs` |
| `css/properties-groups` | colour properties and the properties allowed in `@page` and its margin boxes | stylelint `reference/properties.mjs` |
| `css/shorthands` | each shorthand's longhands and the properties it resets | stylelint `reference/properties.mjs` |
| `css/functions` | colour and math functions | stylelint `reference/functions.mjs` |
| `css/prefixed` | prefixes, and the at-rules, selectors, properties and values a build tool prefixes | stylelint `reference/prefixes.mjs` and `utils/isAutoprefixable.mjs` |
| `css/deprecated` | deprecated at-rules, properties and keywords with their replacements | stylelint `at-rule-no-deprecated`, `property-no-deprecated` and `declaration-property-value-keyword-no-deprecated` |
| `css/descriptors` | the descriptors of each at-rule, and keyword values of some | written for this pack from the CSS specifications |
| `css/property-keywords` | the keywords of single-keyword properties | written for this pack from the CSS specifications |
