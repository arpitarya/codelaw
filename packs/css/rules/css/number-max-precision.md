---
id: css/number-max-precision
title: "At most four decimal places"
languages: [css]
fix: suggest
fix_reason: "rounding changes the value slightly"
stability: preview
options:
  precision: { type: "integer", default: 4, description: "the most decimal places allowed" }
  ignoreUnits: { type: "string[]", default: [], description: "units whose numbers are not checked" }
ported_from: [stylelint/number-max-precision]
since: 0.1.0
---

Numbers have at most four decimal places.

**Why:** Extra precision is noise no screen can render.

```codelaw
native: css/number-max-precision
```

## Examples

### CSS

#### Don't

```css
a {
  line-height: 1.333333;
}
```

#### Do

```css
a {
  line-height: 1.3333;
}
```
