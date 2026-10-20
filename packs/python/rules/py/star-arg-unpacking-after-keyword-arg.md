---
id: py/star-arg-unpacking-after-keyword-arg
title: "No `*args` after a keyword argument"
languages: [python]
fix: manual
fix_reason: "reordering arguments can change which parameter each value binds to"
stability: preview
ported_from: [ruff/B026]
since: 0.1.0
---

In a call, `*args` comes before keyword arguments.

**Why:** `f(x=1, *args)` fills positions before `x`, which reads backwards and can raise `TypeError` for a duplicate argument.

```codelaw
native: py/star-arg-unpacking-after-keyword-arg
```

## How to fix

1. Move `*args` before the keyword arguments and check the call still binds as intended.

## Examples

### Python

#### Don't

```py
run(name="build", *steps)
```

#### Do

```py
run(*steps, name="build")
```
