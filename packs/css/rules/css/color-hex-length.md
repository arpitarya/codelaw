---
id: css/color-hex-length
title: "Short hex colours"
languages: [css, scss]
fix: auto
stability: preview
ported_from: [stylelint/color-hex-length]
since: 0.1.0
---

Hex colours use the short form when there is one: `#fff`, not `#ffffff`.

**Why:** The short form says the same in fewer characters.

```codelaw
native: css/color-hex-length
```

## Examples

### CSS

#### Don't

```css
a {
  color: #ffffff;
}
```

#### Do

```css
a {
  color: #fff;
}
```

### SCSS

#### Don't

```scss
a {
  color: #ffffff;
}
```

#### Do

```scss
a {
  color: #fff;
}
```

## Differences from the source

Ported with stylelint-config-standard's setting, `short`.
