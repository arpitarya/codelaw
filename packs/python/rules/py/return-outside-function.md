---
id: py/return-outside-function
title: "`return` only inside a function"
languages: [python]
fix: manual
fix_reason: "the surrounding code has to be restructured"
stability: preview
ported_from: [ruff/F706]
since: 0.1.0
---

`return` appears only inside a function.

**Why:** At module or class level it is a `SyntaxError`.

```codelaw
native: py/return-outside-function
```

## How to fix

1. Move the code into a function, or remove the `return`.

## Examples

### Python

#### Don't

```py
class Config:
    return None
```

#### Do

```py
class Config:
    def load(self):
        return None
```
