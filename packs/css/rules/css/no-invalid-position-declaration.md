---
id: css/no-invalid-position-declaration
title: "Declarations sit inside rules"
languages: [css, scss]
fix: manual
fix_reason: "the selector it was meant for is the author's call"
stability: stable
ported_from: [stylelint/no-invalid-position-declaration]
since: 0.1.0
---

A declaration sits inside a style rule, in an at-rule nested inside one, or in an at-rule that takes descriptors — never at the top level or directly inside a top-level `@media`.

**Why:** A declaration with no rule around it applies to nothing; the browser drops it.

```codelaw
native: css/no-invalid-position-declaration
```

## How to fix

1. Move the declaration into the style rule it belongs to.

## Examples

### CSS

#### Don't

```css
color: red;
```

#### Do

```css
a {
  color: red;
}
```
### SCSS

#### Don't

```scss
color: red;
```

#### Do

```scss
a {
  color: red;
}
```
