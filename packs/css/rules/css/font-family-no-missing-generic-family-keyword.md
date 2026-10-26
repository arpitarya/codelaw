---
id: css/font-family-no-missing-generic-family-keyword
title: "Font lists end with a generic family"
languages: [css, scss]
fix: manual
fix_reason: "which generic family suits the fonts is a design call"
stability: preview
options:
  ignoreFontFamilies: { type: "string[]", default: [], description: "families that need no generic fallback, such as icon fonts" }
ported_from: [stylelint/font-family-no-missing-generic-family-keyword]
since: 0.1.0
---

A `font-family` list (or the family part of `font`) includes a generic family such as `sans-serif`, `serif` or `monospace`. `@font-face`, values using `var()` and system fonts (`font: caption`) are skipped.

**Why:** When none of the named fonts is available the browser falls back to its default font, which may look nothing like the design.

```codelaw
native: css/font-family-no-missing-generic-family-keyword
```

## How to fix

1. Add the generic family closest to the listed fonts at the end of the list.

## Examples

### CSS

#### Don't

```css
a {
  font-family: Helvetica, Arial;
}
```

#### Do

```css
a {
  font-family: Helvetica, Arial, sans-serif;
}
```
### SCSS

#### Don't

```scss
a {
  font-family: Helvetica, Arial;
}
```

#### Do

```scss
a {
  font-family: Helvetica, Arial, sans-serif;
}
```
