---
id: py/static-key-dict-comprehension
title: "A dict comprehension's key varies"
languages: [python]
fix: manual
fix_reason: "the intended key can't be inferred"
stability: preview
ported_from: [ruff/B035]
since: 0.1.0
---

A dict comprehension's key depends on the loop variables.

**Why:** A constant key gives a dict with one entry, keeping only the last value.

```codelaw
native: py/static-key-dict-comprehension
```

## How to fix

1. Use a key that depends on the loop variable.

## Examples

### Python

#### Don't

```py
sizes = {"size": len(name) for name in names}
```

#### Do

```py
sizes = {name: len(name) for name in names}
```
