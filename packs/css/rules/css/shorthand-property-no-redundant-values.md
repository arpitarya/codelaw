---
id: css/shorthand-property-no-redundant-values
title: "No repeated values in shorthands"
languages: [css, scss]
fix: auto
stability: preview
ported_from: [stylelint/shorthand-property-no-redundant-values]
since: 0.1.0
---

Box shorthands such as `margin`, `padding`, `inset`, `gap` and `border-radius` give no value the shorthand would repeat anyway: `margin: 1px`, not `margin: 1px 1px`.

**Why:** The shorter form says the same thing.

```codelaw
native: css/shorthand-property-no-redundant-values
```

## Examples

### CSS

#### Don't

```css
a {
  margin: 1px 1px;
}
```

#### Do

```css
a {
  margin: 1px;
}
```
### SCSS

#### Don't

```scss
a {
  margin: 1px 1px;
}
```

#### Do

```scss
a {
  margin: 1px;
}
```
