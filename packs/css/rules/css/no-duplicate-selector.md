---
id: css/no-duplicate-selector
title: "No selector declared twice"
languages: [css]
fix: manual
fix_reason: "merging the rules depends on cascade order"
stability: preview
ported_from: [stylelint/no-duplicate-selectors]
since: 0.1.0
---

Two rules in the same at-rule context do not have the same selector list (order and whitespace aside, nesting resolved), and one selector list does not name a selector twice.

**Why:** Split declarations for one selector are hard to follow and often contradict each other.

```codelaw
native: css/no-duplicate-selector
```

## How to fix

1. Merge the declarations of both rules into the first.
2. Check that rules between them did not depend on the order.

## Examples

### CSS

#### Don't

```css
.card {
  color: red;
}

.card {
  margin: 0;
}
```

#### Do

```css
.card {
  color: red;
  margin: 0;
}
```
