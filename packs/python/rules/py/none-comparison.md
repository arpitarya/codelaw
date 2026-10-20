---
id: py/none-comparison
title: "Compare with `None` using `is`"
languages: [python]
fix: suggest
fix_reason: "an object whose `__eq__` treats itself as equal to `None` would compare differently"
stability: preview
ported_from: [ruff/E711]
since: 0.1.0
---

Comparisons with `None` use `is` / `is not`, not `==` / `!=`.

**Why:** `None` is a singleton; `==` calls the other object's `__eq__`, which may not return what you expect.

```codelaw
native: py/none-comparison
```

## Examples

### Python

#### Don't

```py
if user == None:
    login()
```

#### Do

```py
if user is None:
    login()
```
