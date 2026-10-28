---
id: css/named-grid-areas-no-invalid
title: "Named grid areas are rectangles"
languages: [css, scss]
fix: manual
fix_reason: "the intended layout is the author's call"
stability: stable
ported_from: [stylelint/named-grid-areas-no-invalid]
since: 0.1.0
---

In `grid-template-areas` (and `grid-template`, `grid`), every row string has at least one cell, every row has the same number of cells, and each named area forms one filled-in rectangle.

**Why:** The browser drops an invalid `grid-template-areas` declaration, and the grid loses all its named areas.

```codelaw
native: css/named-grid-areas-no-invalid
```

## How to fix

1. Give every row the same number of cells.
2. Make each name cover a rectangle of cells.

## Examples

### CSS

#### Don't

```css
a {
  grid-template-areas:
    "head head"
    "side main main";
}
```

#### Do

```css
a {
  grid-template-areas:
    "head head head"
    "side main main";
}
```
### SCSS

#### Don't

```scss
a {
  grid-template-areas:
    "head head"
    "side main main";
}
```

#### Do

```scss
a {
  grid-template-areas:
    "head head head"
    "side main main";
}
```
