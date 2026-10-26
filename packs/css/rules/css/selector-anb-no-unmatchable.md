---
id: css/selector-anb-no-unmatchable
title: "No An+B selector that matches nothing"
languages: [css, scss]
fix: manual
fix_reason: "the intended position is the author's call"
stability: preview
ported_from: [stylelint/selector-anb-no-unmatchable]
since: 0.1.0
---

`:nth-child()`, `:nth-of-type()` and their `last` forms are not given an argument that is always zero, such as `0`, `0n` or `0n+0`.

**Why:** Positions start at 1, so these selectors never match.

```codelaw
native: css/selector-anb-no-unmatchable
```

## How to fix

1. Use the position that was meant; the first element is `1`.

## Examples

### CSS

#### Don't

```css
li:nth-child(0) {
  color: red;
}
```

#### Do

```css
li:nth-child(1) {
  color: red;
}
```
### SCSS

#### Don't

```scss
li:nth-child(0) {
  color: red;
}
```

#### Do

```scss
li:nth-child(1) {
  color: red;
}
```
