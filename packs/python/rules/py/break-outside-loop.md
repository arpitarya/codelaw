---
id: py/break-outside-loop
title: "`break` only inside a loop"
languages: [python]
fix: manual
fix_reason: "what should happen instead (return, raise) is the author's choice"
stability: preview
ported_from: [ruff/F701]
since: 0.1.0
---

`break` appears only inside the body of a `for` or `while` loop in the same function.

**Why:** Outside a loop it is a `SyntaxError`.

```codelaw
native: py/break-outside-loop
```

## How to fix

1. Use `return` to leave a function, or move the `break` into the loop.

## Examples

### Python

#### Don't

```py
def stop():
    break
```

#### Do

```py
def stop(items):
    for item in items:
        break
```
