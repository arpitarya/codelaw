---
id: css/keyframes-name-pattern
title: "Keyframes names are kebab-case"
languages: [css, scss]
fix: manual
fix_reason: "renaming must update every `animation` that uses the name"
stability: preview
options:
  pattern: { type: "string", default: "^([a-z][a-z0-9]*)(-[a-z0-9]+)*$", description: "a regular expression names must match" }
ported_from: [stylelint/keyframes-name-pattern]
since: 0.1.0
---

The name of each `@keyframes` matches a pattern, kebab-case by default.

**Why:** A consistent naming scheme makes names predictable across a codebase.

```codelaw
native: css/keyframes-name-pattern
```

## How to fix

1. Rename the keyframes to match the pattern.
2. Update every `animation` and `animation-name` that uses it.

## Examples

### CSS

#### Don't

```css
@keyframes fadeIn {
  from {
    opacity: 0;
  }
}
```

#### Do

```css
@keyframes fade-in {
  from {
    opacity: 0;
  }
}
```
### SCSS

#### Don't

```scss
@keyframes fadeIn {
  from {
    opacity: 0;
  }
}
```

#### Do

```scss
@keyframes fade-in {
  from {
    opacity: 0;
  }
}
```
