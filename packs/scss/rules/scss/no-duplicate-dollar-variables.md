---
id: scss/no-duplicate-dollar-variables
title: "No duplicate `$variables`"
languages: [scss]
fix: manual
fix_reason: "which declaration wins is the author's call"
stability: preview
ported_from: [stylelint-scss/no-duplicate-dollar-variables]
since: 0.3.0
---

A `$variable` is declared once per scope. A later `!default` declaration is not a duplicate, and a variable declared in another block is a different one.

**Why:** The second declaration silently replaces the first, so the first is dead code or a mistake.

```codelaw
native: scss/no-duplicate-dollar-variables
```

## How to fix

1. Delete the declaration that should not win.
2. Rename one of them if both are meant.

## Examples

### SCSS

#### Don't

```scss
$gap: 4px;
$gap: 8px;
```

#### Do

```scss
$gap: 4px;
$gap-large: 8px;
```

## Differences from the source

`ignoreDefaults` is always on: a `!default` redeclaration is accepted.
