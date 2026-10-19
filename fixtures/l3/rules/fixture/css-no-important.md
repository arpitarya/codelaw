---
id: fixture/css-no-important
title: L3 fixture rule
languages: [css]
fix: manual
fix_reason: "fixture rule: a human picks the change"
stability: stable
---

A fixture rule for the L3 language matrix (SR-LAW-3); it checks nothing real.

**Why:** every language must be checkable alone and together.

```codelaw
match:
  kind: important
message: "`!important`"
```

## How to fix

1. Change it by hand.

## Examples

### CSS

#### Don't

```css
a { color: red !important; }
```

#### Do

```css
a { color: red; }
```
