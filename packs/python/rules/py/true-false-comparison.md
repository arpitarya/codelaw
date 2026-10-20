---
id: py/true-false-comparison
title: "Don't compare with `True` or `False`"
languages: [python]
fix: suggest
fix_reason: "`x == True` is false for truthy values such as `1` or `[0]`; testing `x` makes them true"
stability: preview
ported_from: [ruff/E712]
since: 0.1.0
---

A condition tests a value directly (`if ready:`, `if not ready:`) instead of comparing it with `True` or `False`.

**Why:** `== True` is noise and is false for truthy values that aren't exactly `True`.

```codelaw
native: py/true-false-comparison
```

## Examples

### Python

#### Don't

```py
if ready == True:
    start()
```

#### Do

```py
if ready:
    start()
```
