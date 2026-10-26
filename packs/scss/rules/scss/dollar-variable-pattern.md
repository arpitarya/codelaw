---
id: scss/dollar-variable-pattern
title: "`$variables` are kebab-case"
languages: [scss]
fix: manual
fix_reason: "renaming must update every use of the variable"
stability: preview
options:
  pattern: { type: "string", default: "^[a-z][a-z0-9]*(-[a-z0-9]+)*$", description: "a regular expression the name, without its `$`, must match" }
ported_from: [stylelint-scss/dollar-variable-pattern]
since: 0.3.0
---

`$variable` names match a pattern, kebab-case by default.

**Why:** A consistent scheme makes names predictable across a codebase.

```codelaw
native: scss/dollar-variable-pattern
```

## How to fix

1. Rename the variable to match the pattern.
2. Rename every use of it.

## Examples

### SCSS

#### Don't

```scss
$FontSize: 16px;
```

#### Do

```scss
$font-size: 16px;
```

## Differences from the source

The pattern applies to declarations, not to the names in an `@each` or a mixin's parameter list.
