---
id: py/raise-without-from-inside-except
title: "Raising in `except` says `from`"
languages: [python]
fix: manual
fix_reason: "whether to chain the original error or hide it is the author's choice"
stability: preview
ported_from: [ruff/B904]
since: 0.1.0
---

A `raise` inside an `except` block that raises a new exception says `from err` or `from None`.

**Why:** Without it, the traceback reads as if a second error happened while handling the first.

```codelaw
native: py/raise-without-from-inside-except
```

## How to fix

1. Bind the caught exception (`except E as err:`).
2. Raise with `from err`, or `from None` to hide the original deliberately.

## Examples

### Python

#### Don't

```py
try:
    value = int(text)
except ValueError:
    raise TypeError("not a number")
```

#### Do

```py
try:
    value = int(text)
except ValueError as err:
    raise TypeError("not a number") from err
```
