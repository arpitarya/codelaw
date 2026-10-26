---
id: scss/no-slash-division
title: "No `/` as division"
languages: [scss]
fix: suggest
fix_reason: "`math.div` needs a `@use` of sass:math, and a slash may be CSS"
stability: preview
since: 0.3.0
---

Division is written `math.div($a, $b)`, not `$a / $b`. A `/` that Sass treats as a plain separator is left alone: `font: 12px/1.5`, `grid-area: 1 / 2`, and anything inside `calc()`.

**Why:** Dart Sass deprecates `/` as division and removes it in 3.0; the slash is ambiguous with CSS separators.

```codelaw
native: scss/no-slash-division
```

## How to fix

1. Add `@use "sass:math";` at the top of the file.
2. Replace `$a / $b` with `math.div($a, $b)`.

## Examples

### SCSS

#### Don't

```scss
.a {
  width: $total / 3;
}
```

#### Do

```scss
@use "sass:math";

.a {
  width: math.div($total, 3);
}
```

## Differences from the source

The check is by operand: a `$variable`, or numbers inside a parenthesised expression. `font: 12px/1.5` is never reported.
