---
id: py/f-string-missing-placeholders
title: "An f-string has a placeholder"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/F541]
since: 0.1.0
---

A string with the `f` prefix contains at least one `{…}` placeholder.

**Why:** Without one, the prefix does nothing and often means a placeholder was forgotten.

```codelaw
native: py/f-string-missing-placeholders
```

## Examples

### Python

#### Don't

```py
print(f"hello")
```

#### Do

```py
print("hello")
```
