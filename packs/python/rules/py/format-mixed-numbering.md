---
id: py/format-mixed-numbering
title: "A `.format` string doesn't mix `{}` and `{0}`"
languages: [python]
fix: manual
fix_reason: "which numbering the author wants has to be chosen"
stability: preview
ported_from: [ruff/F525]
since: 0.1.0
---

A `.format()` string numbers its fields automatically (`{}`) or explicitly (`{0}`), not both.

**Why:** Python raises `ValueError: cannot switch from manual field specification to automatic field numbering`.

```codelaw
native: py/format-mixed-numbering
```

## How to fix

1. Number every field, or none of them.

## Examples

### Python

#### Don't

```py
message = "{0} is {}".format("Ada", 36)
```

#### Do

```py
message = "{0} is {1}".format("Ada", 36)
```
