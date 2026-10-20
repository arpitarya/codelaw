---
id: css/media-feature-range-notation
title: "Range syntax for media features"
languages: [css]
fix: manual
fix_reason: "browsers before 2023 understand only the prefixed form, and codelaw's CSS grammar cannot yet parse the range form, so the engine would refuse the edit"
stability: preview
ported_from: [stylelint/media-feature-range-notation]
since: 0.1.0
---

Range media features use the comparison syntax, `(width >= 600px)`, not `min-` and `max-` prefixes.

**Why:** Comparisons read like the condition they express, and can state exclusive bounds.

```codelaw
native: css/media-feature-range-notation
```

## How to fix

1. Replace `(min-NAME: V)` with `(NAME >= V)` and `(max-NAME: V)` with `(NAME <= V)`.
2. Merge a `min-` and `max-` pair into one range, `(V1 <= NAME <= V2)`.

## Examples

### CSS

#### Don't

```css
@media (min-width: 600px) {
  a {
    color: red;
  }
}
```

#### Do

```css
@media (width >= 600px) {
  a {
    color: red;
  }
}
```

## Differences from the source

Ported with stylelint-config-standard's setting, `context`. Reported without a fix: tree-sitter-css cannot parse the range syntax, so the engine's syntax guard refuses the rewrite.
