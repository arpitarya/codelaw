---
id: py/jump-statement-in-finally
title: "No `return`, `break` or `continue` in `finally`"
languages: [python]
fix: manual
fix_reason: "how errors should propagate has to be decided"
stability: preview
ported_from: [ruff/B012]
since: 0.1.0
---

A `finally` block doesn't leave through `return`, `break` or `continue`.

**Why:** Leaving `finally` that way silently discards any exception in flight.

```codelaw
native: py/jump-statement-in-finally
```

## How to fix

1. Move the `return`, `break` or `continue` out of the `finally` block.
2. Handle the exception explicitly with `except` if it should be swallowed.

## Examples

### Python

#### Don't

```py
def read(path):
    try:
        return load(path)
    finally:
        return None
```

#### Do

```py
def read(path):
    try:
        return load(path)
    finally:
        close(path)
```
