---
id: py/lambda-assignment
title: "Define functions with `def`, not a lambda"
languages: [python]
fix: suggest
fix_reason: "an annotation on the assigned name is dropped by the rewrite"
stability: preview
ported_from: [ruff/E731]
since: 0.1.0
---

A name is bound to a function with `def`, not by assigning a `lambda`.

**Why:** A `def` has a real name in tracebacks, can have a docstring, and reads as a definition.

```codelaw
native: py/lambda-assignment
```

## Examples

### Python

#### Don't

```py
double = lambda x: x * 2
```

#### Do

```py
def double(x):
    return x * 2
```
