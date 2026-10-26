---
id: css/layer-name-pattern
title: "Layer names are kebab-case"
languages: [css, scss]
fix: manual
fix_reason: "renaming must update every use of the layer"
stability: preview
options:
  pattern: { type: "string", default: "^([a-z][a-z0-9]*)([.-][a-z0-9]+)*$", description: "a regular expression names must match" }
ported_from: [stylelint/layer-name-pattern]
since: 0.1.0
---

Cascade layer names, in `@layer` and in `@import … layer()`, match a pattern: kebab-case segments joined by dots, by default.

**Why:** A consistent naming scheme makes names predictable across a codebase.

```codelaw
native: css/layer-name-pattern
```

## How to fix

1. Rename the layer to match the pattern.
2. Update every `@layer` and `@import` that names it.

## Examples

### CSS

#### Don't

```css
@layer baseStyles;
```

#### Do

```css
@layer base-styles;
```
### SCSS

#### Don't

```scss
@layer baseStyles;
```

#### Do

```scss
@layer base-styles;
```
