---
id: py/percent-format-count-mismatch
title: "A `%` format string gets one value per placeholder"
languages: [python]
fix: manual
fix_reason: "which value is missing, or extra, can't be inferred"
stability: preview
ported_from: [ruff/F507]
since: 0.1.0
---

The tuple (or single literal) formatted into a `%` string has as many values as the string has positional placeholders.

**Why:** Too few values raise `TypeError: not enough arguments`; too many raise `TypeError: not all arguments converted`.

```codelaw
native: py/percent-format-count-mismatch
```

## How to fix

1. Add or remove values, or placeholders, until they match.

## Examples

### Python

#### Don't

```py
message = "%s is %s" % ("Ada",)
```

#### Do

```py
message = "%s is %s" % ("Ada", 36)
```
