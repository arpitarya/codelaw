---
id: py/percent-format-star-requires-sequence
title: "A `*` width in a `%` format takes a tuple"
languages: [python]
fix: manual
fix_reason: "the order of the values has to be decided"
stability: preview
ported_from: [ruff/F508]
since: 0.1.0
---

A `%` format string using `*` for a width or precision is applied to a tuple.

**Why:** The `*` takes its value from the next positional argument, which a mapping doesn't have.

```codelaw
native: py/percent-format-star-requires-sequence
```

## How to fix

1. Pass a tuple with the width before the value.

## Examples

### Python

#### Don't

```py
message = "%*d" % {"width": 5, "value": 1}
```

#### Do

```py
message = "%*d" % (5, 1)
```
