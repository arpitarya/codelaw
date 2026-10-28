---
id: css/media-feature-name-no-unknown
title: "No unknown media features"
languages: [css, scss]
fix: manual
fix_reason: "the intended feature is the author's call"
stability: stable
options:
  ignoreMediaFeatureNames: { type: "string[]", default: [], description: "features to accept; `/regex/` for a pattern" }
ported_from: [stylelint/media-feature-name-no-unknown]
since: 0.1.0
---

Every media feature in `@media` and `@custom-media` is one the Media Queries specifications define. Vendor-prefixed features and custom media (`(--name)`) are skipped.

**Why:** A media feature the browser does not know never matches, so the block never applies.

```codelaw
native: css/media-feature-name-no-unknown
```

## How to fix

1. Correct the feature name.

## Examples

### CSS

#### Don't

```css
@media (min-widht: 600px) {
  a {
    color: red;
  }
}
```

#### Do

```css
@media (min-width: 600px) {
  a {
    color: red;
  }
}
```
### SCSS

#### Don't

```scss
@media (min-widht: 600px) {
  a {
    color: red;
  }
}
```

#### Do

```scss
@media (min-width: 600px) {
  a {
    color: red;
  }
}
```
