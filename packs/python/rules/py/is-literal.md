---
id: py/is-literal
title: "Literals are compared with `==`, not `is`"
languages: [python]
fix: suggest
fix_reason: "`==` calls the object's `__eq__`, which can behave differently from identity"
stability: preview
ported_from: [ruff/F632]
since: 0.1.0
---

`is` and `is not` aren't used to compare with a string, bytes or number literal.

**Why:** Whether two equal literals are the same object is an implementation detail; Python warns about it with `SyntaxWarning`.

```codelaw
native: py/is-literal
```

## Examples

### Python

#### Don't

```py
if name is "admin":
    grant()
```

#### Do

```py
if name == "admin":
    grant()
```
