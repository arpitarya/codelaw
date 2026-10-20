---
id: py/ambiguous-function-name
title: "No functions named `l`, `O` or `I`"
languages: [python]
fix: manual
fix_reason: "the new name is the author's choice, and other modules may use the old one"
stability: preview
ported_from: [ruff/E743]
since: 0.1.0
---

Functions aren't named `l`, `O` or `I`.

**Why:** In many fonts they are indistinguishable from `1` and `0`.

```codelaw
native: py/ambiguous-function-name
```

## How to fix

1. Rename the function.
2. Update its callers, here and in modules that import it.

## Examples

### Python

#### Don't

```py
def O():
    return 0
```

#### Do

```py
def origin():
    return 0
```
