---
id: css/color-function-alias-notation
title: "`rgb()` and `hsl()`, not `rgba()` and `hsla()`"
languages: [css, scss]
fix: suggest
fix_reason: "browsers before 2018 ignore an alpha channel in `rgb()`"
stability: preview
ported_from: [stylelint/color-function-alias-notation]
since: 0.1.0
---

Colours use `rgb()` and `hsl()`; their aliases `rgba()` and `hsla()` are not used.

**Why:** The aliases are exactly equivalent today; one spelling is easier to search and read.

```codelaw
native: css/color-function-alias-notation
```

## Examples

### CSS

#### Don't

```css
a {
  color: rgba(0 0 0 / 50%);
}
```

#### Do

```css
a {
  color: rgb(0 0 0 / 50%);
}
```

### SCSS

#### Don't

```scss
a {
  color: rgba(0 0 0 / 50%);
}
```

#### Do

```scss
a {
  color: rgb(0 0 0 / 50%);
}
```

## Differences from the source

Ported with stylelint-config-standard's setting, `without-alpha`.
