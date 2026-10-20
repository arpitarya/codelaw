---
id: py/unused-annotation
title: "Every annotated local is used"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/F842]
since: 0.1.0
---

A local variable annotated without a value (`x: int`) is used somewhere in the function.

**Why:** A local annotation that nothing uses declares nothing; local annotations aren't even evaluated.

```codelaw
native: py/unused-annotation
```

## Examples

### Python

#### Don't

```py
def total():
    count: int
    return 0
```

#### Do

```py
def total():
    return 0
```
