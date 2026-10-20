---
id: py/percent-format-invalid
title: "A `%` format string is complete"
languages: [python]
fix: manual
fix_reason: "the intended placeholder can't be inferred"
stability: preview
ported_from: [ruff/F501]
since: 0.1.0
---

A string formatted with `%` has no unfinished placeholder such as `%(name` or a trailing `%`.

**Why:** Python raises `ValueError` when the string is formatted.

```codelaw
native: py/percent-format-invalid
```

## How to fix

1. Finish the placeholder (`%(name)s`) or escape a literal percent as `%%`.

## Examples

### Python

#### Don't

```py
message = "%(name" % {"name": "Ada"}
```

#### Do

```py
message = "%(name)s" % {"name": "Ada"}
```
