---
id: css/no-irregular-whitespace
title: "No irregular whitespace"
languages: [css, scss]
fix: suggest
fix_reason: "replacing it changes which elements the selector matches"
stability: stable
ported_from: [stylelint/no-irregular-whitespace]
since: 0.1.0
---

Outside comments and strings, the sheet uses only ordinary whitespace: no no-break spaces, zero-width spaces or other Unicode spaces.

**Why:** An irregular space is part of the name around it, so `.menu .item` is one class name and the selector matches nothing.

```codelaw
native: css/no-irregular-whitespace
```

## Examples

### CSS

#### Don't

```css
.menu .item {
  color: red;
}
```

#### Do

```css
.menu .item {
  color: red;
}
```
### SCSS

#### Don't

```scss
.menu .item {
  color: red;
}
```

#### Do

```scss
.menu .item {
  color: red;
}
```
