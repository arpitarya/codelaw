---
id: css/keyframe-block-no-duplicate-selectors
title: "No keyframe selector twice in one `@keyframes`"
languages: [css]
fix: manual
fix_reason: "which keyframe was meant can't be known from the code"
stability: preview
ported_from: [stylelint/keyframe-block-no-duplicate-selectors]
since: 0.1.0
---

A `@keyframes` block names each keyframe selector once.

**Why:** Two blocks for one keyframe merge silently, which is usually a copy-paste slip.

```codelaw
native: css/keyframe-block-no-duplicate-selectors
```

## How to fix

1. Merge the two blocks, or change one selector to the keyframe it was meant for.

## Examples

### CSS

#### Don't

```css
@keyframes fade {
  0% {
    opacity: 0;
  }

  0% {
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

  100% {
    opacity: 1;
  }
}
```
