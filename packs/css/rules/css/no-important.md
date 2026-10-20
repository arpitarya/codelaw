---
id: css/no-important
title: "No `!important`"
languages: [css]
fix: manual
fix_reason: "the selector or layer that should win instead is a design call"
stability: preview
since: 0.1.0
---

No declaration is marked `!important`.

**Why:** `!important` overrides the cascade, and the only way to override it is another `!important`, which escalates.

```codelaw
native: css/no-important
```

## How to fix

1. Find the rule `!important` was overriding.
2. Make this rule win by specificity, order or a cascade layer instead.
3. Remove `!important`.

## Examples

### CSS

#### Don't

```css
a {
  color: red !important;
}
```

#### Do

```css
a.highlight {
  color: red;
}
```
