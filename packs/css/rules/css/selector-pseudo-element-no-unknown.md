---
id: css/selector-pseudo-element-no-unknown
title: "No unknown pseudo-elements"
languages: [css, scss]
fix: manual
fix_reason: "the intended pseudo-element is the author's call"
stability: preview
options:
  ignorePseudoElements: { type: "string[]", default: [], description: "pseudo-elements to accept; `/regex/` for a pattern" }
ported_from: [stylelint/selector-pseudo-element-no-unknown]
since: 0.1.0
---

Every `::` pseudo-element is one the specifications define. Vendor-prefixed ones are skipped.

**Why:** A browser drops a whole rule whose selector has a pseudo-element it does not know.

```codelaw
native: css/selector-pseudo-element-no-unknown
```

## How to fix

1. Correct the pseudo-element name.

## Examples

### CSS

#### Don't

```css
a::befor {
  content: "";
}
```

#### Do

```css
a::before {
  content: "";
}
```
### SCSS

#### Don't

```scss
a::befor {
  content: "";
}
```

#### Do

```scss
a::before {
  content: "";
}
```
