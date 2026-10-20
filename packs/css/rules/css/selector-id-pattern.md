---
id: css/selector-id-pattern
title: "Id selectors are kebab-case"
languages: [css]
fix: manual
fix_reason: "renaming must update the markup and scripts that use the id"
stability: preview
options:
  pattern: { type: "string", default: "^([a-z][a-z0-9]*)(-[a-z0-9]+)*$", description: "a regular expression names must match" }
ported_from: [stylelint/selector-id-pattern]
since: 0.1.0
---

Ids in selectors match a pattern, kebab-case by default.

**Why:** A consistent naming scheme makes names predictable across a codebase.

```codelaw
native: css/selector-id-pattern
```

## How to fix

1. Rename the id to match the pattern.
2. Update the markup and scripts that use it.

## Examples

### CSS

#### Don't

```css
#mainNav {
  color: red;
}
```

#### Do

```css
#main-nav {
  color: red;
}
```
