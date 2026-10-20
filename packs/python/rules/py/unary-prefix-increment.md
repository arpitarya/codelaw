---
id: py/unary-prefix-increment
title: "No `++x` or `--x`"
languages: [python]
fix: manual
fix_reason: "whether an increment or a plain value was meant needs a person"
stability: preview
ported_from: [ruff/B002]
since: 0.1.0
---

Code doesn't write `++x` or `--x`.

**Why:** Python has no increment operator: `++x` is `+(+x)`, which leaves `x` unchanged.

```codelaw
native: py/unary-prefix-increment
```

## How to fix

1. Write `x += 1` (or `x -= 1`) if an increment was meant.

## Examples

### Python

#### Don't

```py
count = 1
++count
```

#### Do

```py
count = 1
count += 1
```
