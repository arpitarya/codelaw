---
id: fixture/py-no-print
title: L3 fixture rule
languages: [python]
fix: manual
fix_reason: "fixture rule: a human picks the change"
stability: stable
---

A fixture rule for the L3 language matrix (SR-LAW-3); it checks nothing real.

**Why:** every language must be checkable alone and together.

```codelaw
match:
  pattern: print($$$ARGS)
message: "print($$$ARGS) in library code"
```

## How to fix

1. Change it by hand.

## Examples

### Python

#### Don't

```py
print(x)
```

#### Do

```py
log.info(x)
```
