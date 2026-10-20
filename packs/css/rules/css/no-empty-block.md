---
id: css/no-empty-block
title: "No empty blocks"
languages: [css]
fix: auto
stability: preview
ported_from: [stylelint/block-no-empty]
since: 0.1.0
---

No rule or at-rule has an empty block. A block holding only a comment is not empty, and `@layer name {}` is allowed because it fixes the layer's place in the cascade order.

**Why:** An empty block does nothing; it is usually left over from an edit or a placeholder that was never filled.

```codelaw
native: css/no-empty-block
```

## Examples

### CSS

#### Don't

```css
.unused {}
a {
  color: red;
}
```

#### Do

```css
a {
  color: red;
}
```

## Differences from the source

`@layer name {}` is not reported: removing it would change the order of cascade layers.
