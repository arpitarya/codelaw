---
id: css/length-zero-no-unit
title: "No units on zero lengths"
languages: [css, scss]
fix: auto
stability: preview
options:
  ignore: { type: "string[]", default: ["custom-properties"], description: "`custom-properties` skips custom property values" }
ported_from: [stylelint/length-zero-no-unit]
since: 0.1.0
---

A zero length is written `0`, without a unit. Custom properties, `line-height`, `flex`, the line height in `font`, math functions and `fr` are skipped, as a unit matters there.

**Why:** `0px`, `0em` and `0` are the same length; the unit is noise.

```codelaw
native: css/length-zero-no-unit
```

## Examples

### CSS

#### Don't

```css
a {
  margin: 0px;
}
```

#### Do

```css
a {
  margin: 0;
}
```
### SCSS

#### Don't

```scss
a {
  margin: 0px;
}
```

#### Do

```scss
a {
  margin: 0;
}
```
