---
id: py/type-comparison
title: "Don't compare types with `==`"
languages: [python]
fix: manual
fix_reason: "`isinstance` accepts subclasses, which may or may not be intended"
stability: preview
ported_from: [ruff/E721]
since: 0.1.0
---

Types are compared with `is` / `is not`, or checked with `isinstance()`, not with `==` or `!=` against `type(x)`.

**Why:** `type(x) == T` ignores subclasses and can be fooled by a metaclass's `__eq__`.

```codelaw
native: py/type-comparison
```

## How to fix

1. Use `isinstance(x, T)` when subclasses should match.
2. Use `type(x) is T` when only the exact type should.

## Examples

### Python

#### Don't

```py
if type(value) == int:
    count(value)
```

#### Do

```py
if isinstance(value, int):
    count(value)
```
