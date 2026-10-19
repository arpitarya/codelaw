---
id: fixture/json-no-null
title: L3 fixture rule
languages: [json]
fix: manual
fix_reason: "fixture rule: a human picks the change"
stability: stable
---

A fixture rule for the L3 language matrix (SR-LAW-3); it checks nothing real.

**Why:** every language must be checkable alone and together.

```codelaw
match:
  kind: "null"
message: "null value"
```

## How to fix

1. Change it by hand.

## Examples

### JSON

#### Don't

```json
{ "a": null }
```

#### Do

```json
{ "a": 1 }
```
