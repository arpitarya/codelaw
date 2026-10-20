---
id: py/format-missing-argument
title: "Every `.format` field has an argument"
languages: [python]
fix: manual
fix_reason: "the missing value has to be supplied"
stability: preview
ported_from: [ruff/F524]
since: 0.1.0
---

A `.format()` call passes an argument for every `{}`, `{0}` and `{name}` field.

**Why:** A missing argument raises `IndexError` or `KeyError` when the string is formatted.

```codelaw
native: py/format-missing-argument
```

## How to fix

1. Pass the missing argument, or remove the field.

## Examples

### Python

#### Don't

```py
message = "{} is {}".format("Ada")
```

#### Do

```py
message = "{} is {}".format("Ada", 36)
```
