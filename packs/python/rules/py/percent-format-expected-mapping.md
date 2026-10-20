---
id: py/percent-format-expected-mapping
title: "Named `%` placeholders get a mapping"
languages: [python]
fix: manual
fix_reason: "the key each value belongs to isn't written anywhere"
stability: preview
ported_from: [ruff/F502]
since: 0.1.0
---

A `%` format string with named placeholders (`%(name)s`) is applied to a mapping, not a tuple.

**Why:** Python raises `TypeError: format requires a mapping`.

```codelaw
native: py/percent-format-expected-mapping
```

## How to fix

1. Pass a dict whose keys are the placeholder names.

## Examples

### Python

#### Don't

```py
message = "%(name)s" % ("Ada",)
```

#### Do

```py
message = "%(name)s" % {"name": "Ada"}
```
