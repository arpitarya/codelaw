---
id: fixture/html-no-marquee
title: L3 fixture rule
languages: [html]
fix: manual
fix_reason: "fixture rule: a human picks the change"
stability: stable
---

A fixture rule for the L3 language matrix (SR-LAW-3); it checks nothing real.

**Why:** every language must be checkable alone and together.

```codelaw
match:
  kind: element
  has: { kind: start_tag, has: { kind: tag_name, regex: ^marquee$ } }
message: "`<marquee>`"
```

## How to fix

1. Change it by hand.

## Examples

### HTML

#### Don't

```html
<marquee>hi</marquee>
```

#### Do

```html
<p>hi</p>
```
