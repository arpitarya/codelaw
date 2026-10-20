---
id: py/ambiguous-class-name
title: "No classes named `l`, `O` or `I`"
languages: [python]
fix: manual
fix_reason: "the new name is the author's choice, and other modules may use the old one"
stability: preview
ported_from: [ruff/E742]
since: 0.1.0
---

Classes aren't named `l`, `O` or `I`.

**Why:** In many fonts they are indistinguishable from `1` and `0`.

```codelaw
native: py/ambiguous-class-name
```

## How to fix

1. Rename the class.
2. Update its uses, here and in modules that import it.

## Examples

### Python

#### Don't

```py
class I:
    pass
```

#### Do

```py
class Item:
    pass
```
