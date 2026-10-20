---
id: css/color-function-notation
title: "Space-separated colour functions"
languages: [css]
fix: suggest
fix_reason: "browsers before 2018 understand only the comma syntax"
stability: preview
ported_from: [stylelint/color-function-notation]
since: 0.1.0
---

`rgb()`, `rgba()`, `hsl()` and `hsla()` use the modern space-separated syntax, with ` / ` before the alpha, not commas.

**Why:** The modern syntax is the one newer colour functions (`lab()`, `oklch()`) share.

```codelaw
native: css/color-function-notation
```

## Examples

### CSS

#### Don't

```css
a {
  color: rgb(0, 0, 0);
}
```

#### Do

```css
a {
  color: rgb(0 0 0);
}
```

## Differences from the source

Ported with stylelint-config-standard's setting, `modern`; values with `var()` inside are skipped.
