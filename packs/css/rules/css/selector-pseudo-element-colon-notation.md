---
id: css/selector-pseudo-element-colon-notation
title: "Pseudo-elements with `::`"
languages: [css, scss]
fix: auto
stability: preview
ported_from: [stylelint/selector-pseudo-element-colon-notation]
since: 0.1.0
---

The pseudo-elements `::before`, `::after`, `::first-line` and `::first-letter` are written with two colons.

**Why:** Two colons tell pseudo-elements from pseudo-classes; the one-colon form exists only for CSS 2 compatibility.

```codelaw
native: css/selector-pseudo-element-colon-notation
```

## Examples

### CSS

#### Don't

```css
a:before {
  content: "";
}
```

#### Do

```css
a::before {
  content: "";
}
```

### SCSS

#### Don't

```scss
a:before {
  content: "";
}
```

#### Do

```scss
a::before {
  content: "";
}
```

## Differences from the source

Ported with stylelint-config-standard's setting, `double`.
