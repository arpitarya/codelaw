---
id: css/container-name-pattern
title: "Container names are kebab-case"
languages: [css, scss]
fix: manual
fix_reason: "renaming must update every use of the name"
stability: stable
options:
  pattern: { type: "string", default: "^(--)?([a-z][a-z0-9]*)(-[a-z0-9]+)*$", description: "a regular expression names must match" }
ported_from: [stylelint/container-name-pattern]
since: 0.1.0
---

Names given in `container-name`, `container` and `@container` match a pattern, kebab-case by default.

**Why:** A consistent naming scheme makes names predictable across a codebase.

```codelaw
native: css/container-name-pattern
```

## How to fix

1. Rename the container to match the pattern.
2. Update every `@container` query that uses it.

## Examples

### CSS

#### Don't

```css
.card {
  container-name: cardGrid;
}
```

#### Do

```css
.card {
  container-name: card-grid;
}
```
### SCSS

#### Don't

```scss
.card {
  container-name: cardGrid;
}
```

#### Do

```scss
.card {
  container-name: card-grid;
}
```
