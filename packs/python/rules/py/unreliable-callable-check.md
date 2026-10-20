---
id: py/unreliable-callable-check
title: "Test for callables with `callable()`"
languages: [python]
fix: suggest
fix_reason: "objects that define `__getattr__` can answer differently to the two tests"
stability: preview
ported_from: [ruff/B004]
since: 0.1.0
---

Code tests whether an object can be called with `callable(x)`, not by looking for a `__call__` attribute.

**Why:** `hasattr(x, "__call__")` can be fooled by `__getattr__` and misses some callables.

```codelaw
native: py/unreliable-callable-check
```

## Examples

### Python

#### Don't

```py
if hasattr(handler, "__call__"):
    handler()
```

#### Do

```py
if callable(handler):
    handler()
```
