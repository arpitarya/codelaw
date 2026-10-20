---
id: py/percent-format-unsupported-character
title: "A `%` format uses known conversions"
languages: [python]
fix: manual
fix_reason: "the intended conversion can't be inferred"
stability: preview
ported_from: [ruff/F509]
since: 0.1.0
---

Each `%` placeholder ends with a conversion Python knows (`s`, `r`, `a`, `d`, `f`, `x`, …).

**Why:** An unknown conversion raises `ValueError: unsupported format character`.

```codelaw
native: py/percent-format-unsupported-character
```

## How to fix

1. Use the conversion you meant, or escape a literal percent as `%%`.

## Examples

### Python

#### Don't

```py
message = "%y" % (1,)
```

#### Do

```py
message = "%s" % (1,)
```
