---
id: py/f-string-docstring
title: "Docstrings aren't f-strings"
languages: [python]
fix: manual
fix_reason: "the docstring's wording has to change to drop the placeholder"
stability: preview
ported_from: [ruff/B021]
since: 0.1.0
---

The first statement of a module, class or function isn't an f-string meant as its docstring.

**Why:** Python treats an f-string there as an expression, so `__doc__` stays empty.

```codelaw
native: py/f-string-docstring
```

## How to fix

1. Remove the `f` prefix and write the text without placeholders.

## Examples

### Python

#### Don't

```py
def greet():
    f"""Say hello to {NAME}."""
```

#### Do

```py
def greet():
    """Say hello to the configured name."""
```
