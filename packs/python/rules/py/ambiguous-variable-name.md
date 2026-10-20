---
id: py/ambiguous-variable-name
title: "No variables named `l`, `O` or `I`"
languages: [python]
fix: manual
fix_reason: "the new name is the author's choice"
stability: preview
ported_from: [ruff/E741]
since: 0.1.0
---

Variables, parameters and import aliases aren't named `l`, `O` or `I`.

**Why:** In many fonts they are indistinguishable from `1` and `0`.

```codelaw
native: py/ambiguous-variable-name
```

## How to fix

1. Rename the variable to a descriptive name.
2. Update its uses.

## Examples

### Python

#### Don't

```py
l = len(items)
```

#### Do

```py
length = len(items)
```
