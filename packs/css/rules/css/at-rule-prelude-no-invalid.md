---
id: css/at-rule-prelude-no-invalid
title: "At-rule preludes follow their grammar"
languages: [css]
fix: manual
fix_reason: "the intended prelude is the author's call"
stability: preview
options:
  ignoreAtRules: { type: "string[]", default: ["media"], description: "at-rules not to check; `@media` is left to `css/media-query-no-invalid`" }
ported_from: [stylelint/at-rule-prelude-no-invalid]
since: 0.1.0
---

The prelude of an at-rule — what comes between its name and its block — has the shape the at-rule needs: a dashed name for `@property`, a valid name for `@keyframes` and `@counter-style`, layer names for `@layer`, a URL for `@import`, nothing for `@font-face`.

**Why:** A browser drops an at-rule whose prelude is invalid, with everything inside it.

```codelaw
native: css/at-rule-prelude-no-invalid
```

## How to fix

1. Rewrite the prelude in the form the at-rule's specification gives.

## Examples

### CSS

#### Don't

```css
@property size {
  syntax: "<length>";
  inherits: false;
  initial-value: 0px;
}
```

#### Do

```css
@property --size {
  syntax: "<length>";
  inherits: false;
  initial-value: 0px;
}
```

## Differences from the source

Checks `@property`, `@font-palette-values`, `@position-try`, `@counter-style`, `@keyframes`, `@layer`, `@import`, `@namespace`, `@charset` and the at-rules that take no prelude; `@supports`, `@container`, `@page` and `@scope` preludes are not checked, as stylelint checks them with the CSSTree grammar codelaw does not ship.
