---
id: css/annotation-no-unknown
title: "No unknown `!` annotations"
languages: [css, scss]
fix: manual
fix_reason: "the intended annotation is the author's call"
stability: stable
options:
  ignoreAnnotations: { type: "string[]", default: [], description: "annotations to accept; `/regex/` for a pattern" }
ported_from: [stylelint/annotation-no-unknown]
since: 0.1.0
---

A declaration ends in no `!` annotation other than `!important`.

**Why:** A misspelt annotation such as `!imprtant` makes the browser drop the whole declaration.

```codelaw
native: css/annotation-no-unknown
```

## How to fix

1. Correct the spelling to `!important`, or remove the annotation.

## Examples

### CSS

#### Don't

```css
a {
  color: green !imprtant;
}
```

#### Do

```css
a {
  color: green !important;
}
```
### SCSS

#### Don't

```scss
a {
  color: green !imprtant;
}
```

#### Do

```scss
a {
  color: green !important;
}
```
