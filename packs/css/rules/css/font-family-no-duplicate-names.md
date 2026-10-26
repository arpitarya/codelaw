---
id: css/font-family-no-duplicate-names
title: "No font family listed twice"
languages: [css, scss]
fix: auto
stability: preview
ported_from: [stylelint/font-family-no-duplicate-names]
since: 0.1.0
---

A `font-family` list (or the family part of `font`) names each family once. A quoted `"serif"` is a family name, not the `serif` keyword, so the two are different.

**Why:** The second mention can never be used, and usually hides a typo.

```codelaw
native: css/font-family-no-duplicate-names
```

## Examples

### CSS

#### Don't

```css
a {
  font-family: Arial, Helvetica, Arial, sans-serif;
}
```

#### Do

```css
a {
  font-family: Arial, Helvetica, sans-serif;
}
```
### SCSS

#### Don't

```scss
a {
  font-family: Arial, Helvetica, Arial, sans-serif;
}
```

#### Do

```scss
a {
  font-family: Arial, Helvetica, sans-serif;
}
```
