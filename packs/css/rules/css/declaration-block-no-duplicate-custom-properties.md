---
id: css/declaration-block-no-duplicate-custom-properties
title: "No custom property declared twice in one block"
languages: [css]
fix: manual
fix_reason: "which value should win can't be known from the code"
stability: preview
ported_from: [stylelint/declaration-block-no-duplicate-custom-properties]
since: 0.1.0
---

A declaration block declares each custom property once.

**Why:** Only the last declaration counts, so the earlier one is dead code or a mistake.

```codelaw
native: css/declaration-block-no-duplicate-custom-properties
```

## How to fix

1. Decide which value the custom property should have.
2. Remove the other declaration.

## Examples

### CSS

#### Don't

```css
a {
  --gap: 4px;
  --gap: 8px;
}
```

#### Do

```css
a {
  --gap: 8px;
}
```
