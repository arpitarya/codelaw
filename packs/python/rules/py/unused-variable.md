---
id: py/unused-variable
title: "Every local variable is used"
languages: [python]
fix: suggest
fix_reason: "dropping the assignment of a plain value is safe, but the right edit for unpacking or `with` targets needs a person"
stability: preview
ported_from: [ruff/F841]
since: 0.1.0
---

A variable assigned in a function (including `with … as` and `except … as` targets) is read somewhere in that function.

**Why:** An unused local is dead code, or a sign the value was meant to be used and isn't.

```codelaw
native: py/unused-variable
```

## Examples

### Python

#### Don't

```py
def handle(request):
    result = compute(request)
    return None
```

#### Do

```py
def handle(request):
    compute(request)
    return None
```
