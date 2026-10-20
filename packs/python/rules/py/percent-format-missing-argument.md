---
id: py/percent-format-missing-argument
title: "Every named `%` placeholder has a value"
languages: [python]
fix: manual
fix_reason: "the value for the missing key has to be supplied"
stability: preview
ported_from: [ruff/F505]
since: 0.1.0
---

A dict literal formatted into a `%` string has a key for every named placeholder.

**Why:** A missing key raises `KeyError` when the string is formatted.

```codelaw
native: py/percent-format-missing-argument
```

## How to fix

1. Add the missing key to the dict, or remove the placeholder.

## Examples

### Python

#### Don't

```py
message = "%(name)s is %(age)s" % {"name": "Ada"}
```

#### Do

```py
message = "%(name)s is %(age)s" % {"name": "Ada", "age": 36}
```
