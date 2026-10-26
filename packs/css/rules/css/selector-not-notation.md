---
id: css/selector-not-notation
title: "One `:not()` with a list"
languages: [css, scss]
fix: suggest
fix_reason: "the merged form has a lower specificity, which can change which rule wins"
stability: preview
ported_from: [stylelint/selector-not-notation]
since: 0.1.0
---

Adjacent `:not()` pseudo-classes are written as one `:not()` with a selector list: `:not(.a, .b)`.

**Why:** Selectors level 4 takes a list in `:not()`; one call reads more simply.

```codelaw
native: css/selector-not-notation
```

## Examples

### CSS

#### Don't

```css
a:not(.active):not(.disabled) {
  color: gray;
}
```

#### Do

```css
a:not(.active, .disabled) {
  color: gray;
}
```

### SCSS

#### Don't

```scss
a:not(.active):not(.disabled) {
  color: gray;
}
```

#### Do

```scss
a:not(.active, .disabled) {
  color: gray;
}
```

## Differences from the source

Ported with stylelint-config-standard's setting, `complex`.
