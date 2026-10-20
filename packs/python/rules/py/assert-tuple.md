---
id: py/assert-tuple
title: "`assert` doesn't test a tuple"
languages: [python]
fix: manual
fix_reason: "removing the parentheses turns a no-op into a check that can now fail"
stability: preview
ported_from: [ruff/F631]
since: 0.1.0
---

An `assert` tests a condition, not a parenthesised `(condition, message)` tuple.

**Why:** A non-empty tuple is always true, so the assert can never fail.

```codelaw
native: py/assert-tuple
```

## How to fix

1. Remove the parentheses so the message becomes the assert's second operand.

## Examples

### Python

#### Don't

```py
assert (count > 0, "count must be positive")
```

#### Do

```py
assert count > 0, "count must be positive"
```
