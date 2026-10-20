---
id: css/selector-pseudo-class-no-unknown
title: "No unknown pseudo-classes"
languages: [css]
fix: manual
fix_reason: "the intended pseudo-class is the author's call"
stability: preview
options:
  ignorePseudoClasses: { type: "string[]", default: [], description: "pseudo-classes to accept; `/regex/` for a pattern" }
ported_from: [stylelint/selector-pseudo-class-no-unknown]
since: 0.1.0
---

Every pseudo-class is one the specifications define, and pseudo-elements are written with `::`. Vendor-prefixed pseudo-classes are skipped; in `@page`, only page pseudo-classes are allowed.

**Why:** A browser drops a whole rule whose selector has a pseudo-class it does not know.

```codelaw
native: css/selector-pseudo-class-no-unknown
```

## How to fix

1. Correct the pseudo-class name.

## Examples

### CSS

#### Don't

```css
a:hovr {
  color: red;
}
```

#### Do

```css
a:hover {
  color: red;
}
```
