---
id: scss/at-mixin-pattern
title: "Mixin names are kebab-case"
languages: [scss]
fix: manual
fix_reason: "renaming must update every `@include` of the mixin"
stability: preview
options:
  pattern: { type: "string", default: "^[a-z][a-z0-9]*(-[a-z0-9]+)*$", description: "a regular expression a mixin name must match" }
ported_from: [stylelint-scss/at-mixin-pattern]
since: 0.3.0
---

Mixin names match a pattern, kebab-case by default.

**Why:** A consistent scheme makes names predictable across a codebase.

```codelaw
native: scss/at-mixin-pattern
```

## How to fix

1. Rename the mixin to match the pattern.
2. Rename every `@include` of it.

## Examples

### SCSS

#### Don't

```scss
@mixin ButtonBase {
  color: red;
}
```

#### Do

```scss
@mixin button-base {
  color: red;
}
```

## Differences from the source

Only the definition is checked.
