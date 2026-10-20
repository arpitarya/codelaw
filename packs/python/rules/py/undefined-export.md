---
id: py/undefined-export
title: "`__all__` names only defined names"
languages: [python]
exclude: ["**/__init__.py"]
fix: manual
fix_reason: "whether the name should be defined or dropped from `__all__` needs a person"
stability: preview
ported_from: [ruff/F822]
since: 0.1.0
---

Every name in a module's `__all__` is defined or imported in that module.

**Why:** `from module import *` raises `AttributeError` on a missing name, and readers trust `__all__` as the module's API.

```codelaw
native: py/undefined-export
```

## How to fix

1. Define or import the name, or remove it from `__all__`.

## Examples

### Python

#### Don't

```py
__all__ = ["load", "save"]


def load():
    pass
```

#### Do

```py
__all__ = ["load", "save"]


def load():
    pass


def save():
    pass
```

## Differences from the source

`__init__.py` files are excluded, as in Ruff, because their `__all__` may list submodules.
