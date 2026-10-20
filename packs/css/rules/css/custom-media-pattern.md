---
id: css/custom-media-pattern
title: "Custom media names are kebab-case"
languages: [css]
fix: manual
fix_reason: "renaming must update every use of the name"
stability: preview
options:
  pattern: { type: "string", default: "^([a-z][a-z0-9]*)(-[a-z0-9]+)*$", description: "a regular expression names must match (without the leading `--`)" }
ported_from: [stylelint/custom-media-pattern]
since: 0.1.0
---

The name of each `@custom-media` (after `--`) matches a pattern, kebab-case by default.

**Why:** A consistent naming scheme makes names predictable across a codebase.

```codelaw
native: css/custom-media-pattern
```

## How to fix

1. Rename the custom media to match the pattern.
2. Update every query that uses it.

## Examples

### CSS

#### Don't

```css
@custom-media --smallViewport (width <= 30em);
```

#### Do

```css
@custom-media --small-viewport (width <= 30em);
```
