---
id: scss/percent-placeholder-pattern
title: "`%placeholder` names are kebab-case"
languages: [scss]
fix: manual
fix_reason: "renaming must update every `@extend` of the placeholder"
stability: preview
options:
  pattern: { type: "string", default: "^[a-z][a-z0-9]*(-[a-z0-9]+)*$", description: "a regular expression a placeholder name, without its `%`, must match" }
ported_from: [stylelint-scss/percent-placeholder-pattern]
since: 0.3.0
---

Placeholder selector names match a pattern, kebab-case by default.

**Why:** A consistent scheme makes names predictable across a codebase.

```codelaw
native: scss/percent-placeholder-pattern
```

## How to fix

1. Rename the placeholder to match the pattern.
2. Rename every `@extend` of it.

## Examples

### SCSS

#### Don't

```scss
%ButtonBase {
  color: red;
}
```

#### Do

```scss
%button-base {
  color: red;
}
```

## Differences from the source

Only definitions are checked.
