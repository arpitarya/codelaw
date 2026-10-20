---
id: py/all-sorted
title: "`__all__` is sorted"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/RUF022]
since: 0.1.0
---

A module's `__all__` lists its names in isort's order: CONSTANTS, then Classes, then other names, each group in natural order.

**Why:** A sorted `__all__` is easy to scan and to keep in sync with the module.

```codelaw
native: py/all-sorted
```

## Examples

### Python

#### Don't

```py
__all__ = ["save", "Loader", "load", "VERSION"]
```

#### Do

```py
__all__ = ["VERSION", "Loader", "load", "save"]
```
