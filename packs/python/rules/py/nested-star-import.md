---
id: py/nested-star-import
title: "`import *` only at module level"
languages: [python]
fix: manual
fix_reason: "the names to import must be chosen by reading the function"
stability: preview
ported_from: [ruff/F406]
since: 0.1.0
---

`from module import *` appears only at the top level of a module.

**Why:** Inside a function or class a star import is a `SyntaxError` in Python 3.

```codelaw
native: py/nested-star-import
```

## How to fix

1. Replace the star with the names the function uses.

## Examples

### Python

#### Don't

```py
def paths():
    from os.path import *
    return join("a", "b")
```

#### Do

```py
def paths():
    from os.path import join
    return join("a", "b")
```
