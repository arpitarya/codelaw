---
id: css/at-rule-descriptor-no-unknown
title: "No unknown descriptors inside at-rules"
languages: [css, scss]
fix: manual
fix_reason: "whether to remove or move the declaration is the author's call"
stability: stable
ported_from: [stylelint/at-rule-descriptor-no-unknown]
since: 0.1.0
---

Inside `@font-face`, `@counter-style`, `@property`, `@font-palette-values`, `@page`, `@view-transition`, `@color-profile` and `@font-feature-values`, every declaration is a descriptor that at-rule defines.

**Why:** An at-rule ignores descriptors it does not define, so a misplaced property silently does nothing.

```codelaw
native: css/at-rule-descriptor-no-unknown
```

## How to fix

1. Check the descriptor name against the at-rule's specification.
2. Fix its spelling, or move the declaration into a style rule where it applies.

## Examples

### CSS

#### Don't

```css
@font-face {
  font-family: "Inter";
  src: url("inter.woff2");
  color: red;
}
```

#### Do

```css
@font-face {
  font-family: "Inter";
  src: url("inter.woff2");
}
```
### SCSS

#### Don't

```scss
@font-face {
  font-family: "Inter";
  src: url("inter.woff2");
  color: red;
}
```

#### Do

```scss
@font-face {
  font-family: "Inter";
  src: url("inter.woff2");
}
```
