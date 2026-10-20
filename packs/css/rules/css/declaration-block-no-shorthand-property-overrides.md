---
id: css/declaration-block-no-shorthand-property-overrides
title: "No shorthand after a longhand it resets"
languages: [css]
fix: manual
fix_reason: "which value is meant can't be known from the code"
stability: preview
ported_from: [stylelint/declaration-block-no-shorthand-property-overrides]
since: 0.1.0
---

A shorthand property does not follow, in the same block, a longhand it covers.

**Why:** The shorthand resets the longhand declared before it, so the longhand never applies.

```codelaw
native: css/declaration-block-no-shorthand-property-overrides
```

## How to fix

1. Move the longhand after the shorthand, or fold its value into the shorthand.

## Examples

### CSS

#### Don't

```css
a {
  padding-left: 10px;
  padding: 20px;
}
```

#### Do

```css
a {
  padding: 20px;
  padding-left: 10px;
}
```
