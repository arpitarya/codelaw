---
id: py/no-mutable-default
title: "No mutable default arguments"
languages: [python]
fix: suggest
fix_reason: "callers that relied on the shared object, or passed `None` on purpose, behave differently"
stability: preview
ported_from: [ruff/B006]
since: 0.1.0
---

A parameter's default isn't a list, dict or set (literal, comprehension or constructor call).

**Why:** The default is created once, when the function is defined, and every call shares and mutates the same object.

```codelaw
native: py/no-mutable-default
```

## Examples

### Python

#### Don't

```py
def append(item, items=[]):
    items.append(item)
    return items
```

#### Do

```py
def append(item, items=None):
    if items is None:
        items = []
    items.append(item)
    return items
```

## Differences from the source

The fix is offered only for unannotated parameters whose default doesn't read the parameter's own name; Ruff also rewrites an annotation to `T | None`.
