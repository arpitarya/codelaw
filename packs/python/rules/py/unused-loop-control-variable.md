---
id: py/unused-loop-control-variable
title: "Every loop variable is used in the loop"
languages: [python]
fix: suggest
fix_reason: "the rename is offered only when the name isn't used anywhere else; otherwise a person decides"
stability: preview
ported_from: [ruff/B007]
since: 0.1.0
---

A `for` loop's target names are read in the loop body, or start with `_` to say they aren't needed.

**Why:** An unused loop variable often means the body uses the wrong name.

```codelaw
native: py/unused-loop-control-variable
```

## Examples

### Python

#### Don't

```py
for i in range(3):
    retry()
```

#### Do

```py
for _i in range(3):
    retry()
```
