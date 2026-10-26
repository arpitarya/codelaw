---
id: css/keyframe-declaration-no-important
title: "No `!important` in keyframes"
languages: [css, scss]
fix: suggest
fix_reason: "removing `!important` makes an ignored declaration apply, which changes the animation"
stability: preview
ported_from: [stylelint/keyframe-declaration-no-important]
since: 0.1.0
---

No declaration inside a `@keyframes` block is marked `!important`.

**Why:** Browsers ignore a keyframe declaration marked `!important`, so the property is not animated.

```codelaw
native: css/keyframe-declaration-no-important
```

## Examples

### CSS

#### Don't

```css
@keyframes fade {
  from {
    opacity: 0 !important;
  }

  to {
    opacity: 1;
  }
}
```

#### Do

```css
@keyframes fade {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}
```
### SCSS

#### Don't

```scss
@keyframes fade {
  from {
    opacity: 0 !important;
  }

  to {
    opacity: 1;
  }
}
```

#### Do

```scss
@keyframes fade {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}
```
