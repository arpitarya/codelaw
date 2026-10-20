---
id: py/invalid-print-syntax
title: "No Python 2 `print >>`"
languages: [python]
fix: manual
fix_reason: "the rewrite depends on how the Python 2 arguments were meant"
stability: preview
ported_from: [ruff/F633]
since: 0.1.0
---

`print` is called as a function; the Python 2 form `print >> stream, value` isn't used.

**Why:** In Python 3 it shifts the `print` function right, raising `TypeError`.

```codelaw
native: py/invalid-print-syntax
```

## How to fix

1. Rewrite as `print(value, file=stream)`.

## Examples

### Python

#### Don't

```py
import sys

print >> sys.stderr, "error"
```

#### Do

```py
import sys

print("error", file=sys.stderr)
```
