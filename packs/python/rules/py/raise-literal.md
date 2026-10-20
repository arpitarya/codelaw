---
id: py/raise-literal
title: "Raise exceptions, not literals"
languages: [python]
fix: manual
fix_reason: "the exception class to raise is the author's choice"
stability: preview
ported_from: [ruff/B016]
since: 0.1.0
---

`raise` is given an exception, not a string or number literal.

**Why:** Raising a literal is a `TypeError` in Python 3.

```codelaw
native: py/raise-literal
```

## How to fix

1. Raise an exception class with the literal as its message.

## Examples

### Python

#### Don't

```py
raise "failed"
```

#### Do

```py
raise RuntimeError("failed")
```
