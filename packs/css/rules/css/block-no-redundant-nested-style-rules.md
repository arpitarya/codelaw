---
id: css/block-no-redundant-nested-style-rules
title: "No `& { \u2026 }` wrapper"
languages: [css, scss]
fix: suggest
fix_reason: "declarations moved out of the wrapper change place relative to the nested rules around it"
stability: stable
ported_from: [stylelint/block-no-redundant-nested-style-rules]
since: 0.1.0
---

A nested style rule whose selector is only `&` is not used: its declarations go straight in the parent block.

**Why:** `& { … }` selects the same elements as its parent, so the wrapper only adds nesting.

```codelaw
native: css/block-no-redundant-nested-style-rules
```

## Examples

### CSS

#### Don't

```css
a {
  & {
    color: red;
  }
}
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
a {
  & {
    color: red;
  }
}
```

#### Do

```scss
a {
  color: red;
}
```
