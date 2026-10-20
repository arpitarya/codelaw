---
id: css/custom-property-pattern
title: "Custom property names are kebab-case"
languages: [css]
fix: manual
fix_reason: "renaming must update every use of the name, also in scripts"
stability: preview
options:
  pattern: { type: "string", default: "^([a-z][a-z0-9]*)(-[a-z0-9]+)*$", description: "a regular expression names must match (without the leading `--`)" }
ported_from: [stylelint/custom-property-pattern]
since: 0.1.0
---

Custom property names (after `--`), in declarations, `var()` and `@property`, match a pattern, kebab-case by default.

**Why:** A consistent naming scheme makes names predictable across a codebase.

```codelaw
native: css/custom-property-pattern
```

## How to fix

1. Rename the custom property to match the pattern.
2. Update every `var()` and script that reads it.

## Examples

### CSS

#### Don't

```css
:root {
  --brandColor: red;
}
```

#### Do

```css
:root {
  --brand-color: red;
}
```
