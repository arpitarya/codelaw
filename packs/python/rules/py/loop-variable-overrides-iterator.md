---
id: py/loop-variable-overrides-iterator
title: "A loop variable doesn't replace what it iterates"
languages: [python]
fix: manual
fix_reason: "the new name is the author's choice"
stability: preview
ported_from: [ruff/B020]
since: 0.1.0
---

A `for` loop's target doesn't reuse a name read in the iterable expression.

**Why:** After the loop the name holds the last item, not the collection, which breaks later uses.

```codelaw
native: py/loop-variable-overrides-iterator
```

## How to fix

1. Give the loop variable its own name.
2. Update its uses in the loop body.

## Examples

### Python

#### Don't

```py
for items in items:
    print(items)
```

#### Do

```py
for item in items:
    print(item)
```
