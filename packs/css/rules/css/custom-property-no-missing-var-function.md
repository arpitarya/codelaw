---
id: css/custom-property-no-missing-var-function
title: "Custom properties are read with `var()`"
languages: [css, scss]
fix: suggest
fix_reason: "wrapping in `var()` makes a dropped declaration apply, which changes the page"
stability: preview
ported_from: [stylelint/custom-property-no-missing-var-function]
since: 0.1.0
---

A value names a custom property defined in the same sheet only inside `var()`.

**Why:** Without `var()` the value is the identifier `--name`, which is invalid for most properties, so the declaration is dropped.

```codelaw
native: css/custom-property-no-missing-var-function
```

## Examples

### CSS

#### Don't

```css
:root {
  --brand: red;
}

a {
  color: --brand;
}
```

#### Do

```css
:root {
  --brand: red;
}

a {
  color: var(--brand);
}
```
### SCSS

#### Don't

```scss
:root {
  --brand: red;
}

a {
  color: --brand;
}
```

#### Do

```scss
:root {
  --brand: red;
}

a {
  color: var(--brand);
}
```
