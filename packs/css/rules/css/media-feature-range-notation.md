---
id: css/media-feature-range-notation
title: "Range syntax for media features"
languages: [css]
fix: auto
stability: preview
ported_from: [stylelint/media-feature-range-notation]
since: 0.1.0
---

Range media features use the comparison syntax, `(width >= 600px)`, not `min-` and `max-` prefixes. The fix rewrites each prefixed feature on its own: `(min-NAME: V)` becomes `(NAME >= V)` and `(max-NAME: V)` becomes `(NAME <= V)`; a `min-` and `max-` pair becomes two ranges, which can be merged by hand into `(V1 <= NAME <= V2)`.

**Why:** Comparisons read like the condition they express, and can state exclusive bounds.

```codelaw
native: css/media-feature-range-notation
```

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

Ported with stylelint-config-standard's setting, `context`. The fix does not merge a `min-` and `max-` pair into one range.
