---
id: py/continue-outside-loop
title: "`continue` only inside a loop"
languages: [python]
fix: manual
fix_reason: "what should happen instead is the author's choice"
stability: preview
ported_from: [ruff/F702]
since: 0.1.0
---

`continue` appears only inside the body of a `for` or `while` loop in the same function.

**Why:** Outside a loop it is a `SyntaxError`.

```codelaw
native: py/continue-outside-loop
```

## How to fix

1. Move the `continue` into the loop, or restructure the code.

## Examples

### Python

#### Don't

```py
def skip():
    continue
```

#### Do

```py
def skip(items):
    for item in items:
        continue
```
