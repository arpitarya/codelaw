---
id: css/keyframe-selector-notation
title: "Percentages for keyframe selectors"
languages: [css, scss]
fix: auto
stability: preview
ported_from: [stylelint/keyframe-selector-notation]
since: 0.1.0
---

Keyframe selectors are percentages (`0%`, `100%`) unless every selector of the `@keyframes` is a keyword (`from`, `to`).

**Why:** Mixing `from` with `50%` makes the timeline harder to read.

```codelaw
native: css/keyframe-selector-notation
```

## Examples

### CSS

#### Don't

```css
@keyframes fade {
  from {
    opacity: 0;
  }

  50% {
    opacity: 0.5;
  }

  to {
    opacity: 1;
  }
}
```

#### Do

```css
@keyframes fade {
  0% {
    opacity: 0;
  }

  50% {
    opacity: 0.5;
  }

  100% {
    opacity: 1;
  }
}
```

### SCSS

#### Don't

```scss
@keyframes fade {
  from {
    opacity: 0;
  }

  50% {
    opacity: 0.5;
  }

  to {
    opacity: 1;
  }
}
```

#### Do

```scss
@keyframes fade {
  0% {
    opacity: 0;
  }

  50% {
    opacity: 0.5;
  }

  100% {
    opacity: 1;
  }
}
```

## Differences from the source

Ported with stylelint-config-standard's setting, `percentage-unless-within-keyword-only-block`.
