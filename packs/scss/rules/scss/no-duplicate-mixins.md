---
id: scss/no-duplicate-mixins
title: "No duplicate mixins"
languages: [scss]
fix: manual
fix_reason: "which definition should stay is the author's call"
stability: preview
ported_from: [stylelint-scss/no-duplicate-mixins]
since: 0.3.0
---

A mixin is defined once per scope.

**Why:** The later definition replaces the earlier one; every `@include` before it silently gets the other body.

```codelaw
native: scss/no-duplicate-mixins
```

## How to fix

1. Delete or rename one of the definitions.

## Examples

### SCSS

#### Don't

```scss
@mixin shadow {
  box-shadow: none;
}
@mixin shadow {
  box-shadow: 0 0 1px;
}
```

#### Do

```scss
@mixin shadow {
  box-shadow: none;
}
@mixin shadow-soft {
  box-shadow: 0 0 1px;
}
```

## Differences from the source

Two scopes may use the same name.
