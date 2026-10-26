---
id: css/nesting-selector-no-missing-scoping-root
title: "Nesting selectors have a parent"
languages: [css, scss]
fix: manual
fix_reason: "the intended parent selector is the author's call"
stability: preview
ported_from: [stylelint/nesting-selector-no-missing-scoping-root]
since: 0.1.0
---

The nesting selector `&` is used only inside a style rule (or under `@scope`), where it stands for the parent selector.

**Why:** At the top level `&` matches the root element, which is almost never what was meant.

```codelaw
native: css/nesting-selector-no-missing-scoping-root
```

## How to fix

1. Nest the rule inside the rule `&` should refer to, or replace `&` with that selector.

## Examples

### CSS

#### Don't

```css
& .item {
  color: red;
}
```

#### Do

```css
.list {
  & .item {
    color: red;
  }
}
```
### SCSS

#### Don't

```scss
& .item {
  color: red;
}
```

#### Do

```scss
.list {
  & .item {
    color: red;
  }
}
```
