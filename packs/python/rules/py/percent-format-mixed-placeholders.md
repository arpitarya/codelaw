---
id: py/percent-format-mixed-placeholders
title: "A `%` format string doesn't mix positional and named placeholders"
languages: [python]
fix: manual
fix_reason: "which style the string should use is the author's choice"
stability: preview
ported_from: [ruff/F506]
since: 0.1.0
---

A `%` format string uses either positional placeholders (`%s`) or named ones (`%(name)s`), not both.

**Why:** Python can't fill both from one right-hand side and raises `TypeError`.

```codelaw
native: py/percent-format-mixed-placeholders
```

## How to fix

1. Name every placeholder, or make every placeholder positional.

## Examples

### Python

#### Don't

```py
message = "%(name)s is %s" % {"name": "Ada"}
```

#### Do

```py
message = "%(name)s is %(age)s" % {"name": "Ada", "age": 36}
```
