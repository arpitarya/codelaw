---
id: css/media-feature-name-no-vendor-prefix
title: "No vendor-prefixed media features"
languages: [css]
fix: manual
fix_reason: "the value needs converting to a `dppx` resolution"
stability: preview
ported_from: [stylelint/media-feature-name-no-vendor-prefix]
since: 0.1.0
---

Media queries use no vendor-prefixed `device-pixel-ratio` feature.

**Why:** The standard `resolution` feature covers it in every current browser.

```codelaw
native: css/media-feature-name-no-vendor-prefix
```

## How to fix

1. Replace the feature with `min-resolution`, `max-resolution` or `resolution`, and its value `n` with `ndppx`.

## Examples

### CSS

#### Don't

```css
@media (-webkit-min-device-pixel-ratio: 2) {
  a {
    color: red;
  }
}
```

#### Do

```css
@media (min-resolution: 2dppx) {
  a {
    color: red;
  }
}
```

## Differences from the source

Reported without a fix: stylelint only drops the prefix, which leaves a non-standard feature.
