---
id: css/hue-degree-notation
title: "Hues are angles"
languages: [css, scss]
fix: auto
stability: stable
ported_from: [stylelint/hue-degree-notation]
since: 0.1.0
---

The hue of `hsl()`, `hwb()`, `lch()` and `oklch()` is written as an angle (`120deg`), not a bare number.

**Why:** An explicit unit makes the value's meaning plain.

```codelaw
native: css/hue-degree-notation
```

## Examples

### CSS

#### Don't

```css
a {
  color: hsl(120 60% 50%);
}
```

#### Do

```css
a {
  color: hsl(120deg 60% 50%);
}
```

### SCSS

#### Don't

```scss
a {
  color: hsl(120 60% 50%);
}
```

#### Do

```scss
a {
  color: hsl(120deg 60% 50%);
}
```

## Differences from the source

Ported with stylelint-config-standard's setting, `angle`.
