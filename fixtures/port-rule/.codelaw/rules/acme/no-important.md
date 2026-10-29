---
id: acme/no-important
title: "Declarations don't use !important"
languages: [css, scss]
fix: manual
fix_reason: "dropping !important changes which declaration wins the cascade; the selectors involved need the author"
stability: preview
ported_from: [stylelint/declaration-no-important]
since: 0.4.0
---

No declaration ends in `!important`, wherever it is — in a rule, a nested rule, an at-rule's block.
Known gap against the source rule: a `!` and `important` written apart (`! important`) does not parse as
a declaration in codelaw's CSS grammar, so it is not reported here.

**Why:** `!important` wins over every normal declaration whatever its selector, so the next change has to
answer it with another `!important`. Ported from stylelint's `declaration-no-important` by the
`codelaw-port-rule` skill: if a first-party pack ships a rule with the same `ported_from:`, turn that one
on and delete this file.

```codelaw
match:
  kind: important
  inside: { kind: declaration }
message: "`!important` overrides the cascade: raise the selector's specificity or reorder the rules instead"
```

## How to fix

1. Find the declaration the `!important` was written to beat (the browser's computed-style panel shows it).
2. Make this selector the more specific one, or move it after the other, or drop the other declaration.
3. Remove `!important` and check the element still gets the intended value.

## Examples

### CSS

#### Don't

```css
.notice {
  color: crimson !important;
}
```

#### Do

```css
.panel .notice {
  color: crimson;
}
```

### SCSS

#### Don't

```scss
.card {
  .title {
    margin: 0 !important;
  }
}
```

#### Do

```scss
.card {
  .title {
    margin: 0;
  }
}
```
