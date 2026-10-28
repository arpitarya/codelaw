---
id: css/at-rule-descriptor-value-no-unknown
title: "No unknown keyword values for at-rule descriptors"
languages: [css, scss]
fix: manual
fix_reason: "the intended value is the author's call"
stability: stable
ported_from: [stylelint/at-rule-descriptor-value-no-unknown]
since: 0.1.0
---

Descriptors that take a fixed set of keywords — `font-display`, `system`, `inherits`, `navigation` and the like — use one of them, and the `syntax` of `@property` is a string.

**Why:** The browser drops a descriptor with a value it does not recognise, so the at-rule falls back to its default.

```codelaw
native: css/at-rule-descriptor-value-no-unknown
```

## How to fix

1. Replace the value with one the descriptor accepts.

## Examples

### CSS

#### Don't

```css
@font-face {
  font-family: "Inter";
  src: url("inter.woff2");
  font-display: quick;
}
```

#### Do

```css
@font-face {
  font-family: "Inter";
  src: url("inter.woff2");
  font-display: swap;
}
```

### SCSS

#### Don't

```scss
@font-face {
  font-family: "Inter";
  src: url("inter.woff2");
  font-display: quick;
}
```

#### Do

```scss
@font-face {
  font-family: "Inter";
  src: url("inter.woff2");
  font-display: swap;
}
```

## Differences from the source

Checks keyword-valued descriptors and the `@property` `syntax` type only; stylelint validates every descriptor value against the full CSSTree grammar, whose data derives from MDN data (CC0), which codelaw does not ship.
