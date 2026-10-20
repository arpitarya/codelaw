---
id: py/percent-format-expected-sequence
title: "Positional `%` placeholders get a tuple"
languages: [python]
fix: manual
fix_reason: "the order of the values isn't written anywhere"
stability: preview
ported_from: [ruff/F503]
since: 0.1.0
---

A `%` format string with two or more positional placeholders is applied to a tuple, not a dict.

**Why:** A dict is a single value, so Python raises `TypeError: not enough arguments`.

```codelaw
native: py/percent-format-expected-sequence
```

## How to fix

1. Pass a tuple with one value per placeholder, or switch to named placeholders.

## Examples

### Python

#### Don't

```py
message = "%s and %s" % {"a": 1, "b": 2}
```

#### Do

```py
message = "%s and %s" % (1, 2)
```
