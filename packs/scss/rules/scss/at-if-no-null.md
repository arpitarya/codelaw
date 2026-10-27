---
id: scss/at-if-no-null
title: "No comparing to `null` in `@if`"
languages: [scss]
fix: auto
stability: preview
ported_from: [stylelint-scss/at-if-no-null]
since: 0.3.0
---

`@if` tests a value directly: `@if $x`, `@if not $x`, not `@if $x != null` or `@if $x == null`.

**Why:** Every value other than `null` and `false` is truthy in Sass, so testing the value says the same thing in fewer words.

```codelaw
native: scss/at-if-no-null
```

## Examples

### SCSS

#### Don't

```scss
@if $x != null {
  a: b;
}
```

#### Do

```scss
@if $x {
  a: b;
}
```

## Differences from the source

`$x != null` is fixed to `$x` and `$x == null` to `not $x`. A comparison that is only part of a condition is reported without a fix.
