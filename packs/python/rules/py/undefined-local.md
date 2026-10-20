---
id: py/undefined-local
title: "A local isn't read before the function assigns it"
languages: [python]
fix: manual
fix_reason: "whether to declare it `global`, `nonlocal` or rename the local needs a person"
stability: preview
ported_from: [ruff/F823]
since: 0.1.0
---

A function doesn't read a name before assigning it, when that name also exists in an enclosing scope.

**Why:** Assigning makes the name local for the whole function, so the earlier read raises `UnboundLocalError` instead of seeing the outer value.

```codelaw
native: py/undefined-local
```

## How to fix

1. Declare the name `global` or `nonlocal` if the outer one is meant.
2. Otherwise rename the local variable.

## Examples

### Python

#### Don't

```py
count = 0


def increment():
    count += 1
```

#### Do

```py
count = 0


def increment():
    global count
    count += 1
```
