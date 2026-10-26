---
id: css/function-calc-no-unspaced-operator
title: "Spaces around `+` and `-` in math functions"
languages: [css, scss]
fix: suggest
fix_reason: "the fix makes an invalid, dropped declaration apply, which changes the page"
stability: preview
ported_from: [stylelint/function-calc-no-unspaced-operator]
since: 0.1.0
---

Inside `calc()` and the other math functions, `+` and `-` have whitespace on both sides.

**Why:** Without the spaces `-10px` reads as a negative number and `100%-10px` as one token, so the expression is invalid and the declaration is dropped.

```codelaw
native: css/function-calc-no-unspaced-operator
```

## Examples

### CSS

#### Don't

```css
a {
  width: calc(100%-10px);
}
```

#### Do

```css
a {
  width: calc(100% - 10px);
}
```

### SCSS

#### Don't

```scss
a {
  width: calc(100%-10px);
}
```

#### Do

```scss
a {
  width: calc(100% - 10px);
}
```

## Differences from the source

Whitespace other than a single space around an operator is left to the formatter.
