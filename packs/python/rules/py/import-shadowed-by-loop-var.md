---
id: py/import-shadowed-by-loop-var
title: "A loop variable doesn't reuse an imported name"
languages: [python]
fix: manual
fix_reason: "the new loop variable name is a choice the author has to make"
stability: preview
ported_from: [ruff/F402]
since: 0.1.0
---

A `for` loop's target doesn't rebind a name that an import in the same scope bound.

**Why:** After the loop the name no longer refers to the module, so later uses break in ways that are hard to trace.

```codelaw
native: py/import-shadowed-by-loop-var
```

## How to fix

1. Rename the loop variable.
2. Update its uses inside the loop body.

## Examples

### Python

#### Don't

```py
import path

for path in ["a.txt", "b.txt"]:
    print(path)
```

#### Do

```py
import path

for name in ["a.txt", "b.txt"]:
    print(path.join(name))
```
