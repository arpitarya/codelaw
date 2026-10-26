---
id: css/declaration-property-value-no-unknown
title: "Keyword-valued properties use a known keyword"
languages: [css, scss]
fix: manual
fix_reason: "the intended value is the author's call"
stability: preview
ported_from: [stylelint/declaration-property-value-no-unknown]
since: 0.1.0
---

A property whose value is one keyword from a fixed set — `position`, `display`-like `float` and `clear`, `visibility`, `box-sizing`, `overflow`, border styles and others — is given one of those keywords or a CSS-wide keyword.

**Why:** The browser drops a declaration whose value it does not recognise.

```codelaw
native: css/declaration-property-value-no-unknown
```

## How to fix

1. Replace the value with one the property accepts.

## Examples

### CSS

#### Don't

```css
a {
  position: centre;
}
```

#### Do

```css
a {
  position: relative;
}
```

### SCSS

#### Don't

```scss
a {
  position: centre;
}
```

#### Do

```scss
a {
  position: relative;
}
```

## Differences from the source

Checks single-keyword values of about forty keyword-valued properties, from a table written for this pack; stylelint validates every value against the full CSSTree grammar, whose data derives from MDN data (CC0), which codelaw does not ship.
