---
id: py/format-invalid
title: "A `.format` string is well formed"
languages: [python]
fix: manual
fix_reason: "whether the brace is a field or literal text has to be decided"
stability: preview
ported_from: [ruff/F521]
since: 0.1.0
---

A string literal passed through `.format()` has balanced braces; literal braces are written `{{` and `}}`.

**Why:** A stray brace raises `ValueError` when the string is formatted.

```codelaw
native: py/format-invalid
```

## How to fix

1. Close the field, or double a literal brace (`{{`, `}}`).

## Examples

### Python

#### Don't

```py
message = "{".format(1)
```

#### Do

```py
message = "{}".format(1)
```
