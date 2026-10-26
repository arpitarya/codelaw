---
id: css/lightness-notation
title: "Lightness is a percentage"
languages: [css, scss]
fix: auto
stability: preview
ported_from: [stylelint/lightness-notation]
since: 0.1.0
---

The lightness of `lab()`, `lch()`, `oklab()` and `oklch()` is written as a percentage.

**Why:** The functions use different number scales for lightness (0–1 and 0–100); a percentage reads the same in all of them.

```codelaw
native: css/lightness-notation
```

## Examples

### CSS

#### Don't

```css
a {
  color: oklch(0.7 0.1 200deg);
}
```

#### Do

```css
a {
  color: oklch(70% 0.1 200deg);
}
```

### SCSS

#### Don't

```scss
a {
  color: oklch(0.7 0.1 200deg);
}
```

#### Do

```scss
a {
  color: oklch(70% 0.1 200deg);
}
```

## Differences from the source

Ported with stylelint-config-standard's setting, `percentage`.
