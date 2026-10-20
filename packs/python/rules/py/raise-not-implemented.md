---
id: py/raise-not-implemented
title: "Raise `NotImplementedError`, not `NotImplemented`"
languages: [python]
fix: suggest
fix_reason: "callers catching the `TypeError` the broken code raised would now see a different exception"
stability: preview
ported_from: [ruff/F901]
since: 0.1.0
---

`raise` uses `NotImplementedError`, not the `NotImplemented` constant.

**Why:** `NotImplemented` is a value for binary operators; raising it is a `TypeError`.

```codelaw
native: py/raise-not-implemented
```

## Examples

### Python

#### Don't

```py
class Shape:
    def area(self):
        raise NotImplemented
```

#### Do

```py
class Shape:
    def area(self):
        raise NotImplementedError
```
