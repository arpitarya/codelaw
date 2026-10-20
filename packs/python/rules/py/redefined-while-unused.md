---
id: py/redefined-while-unused
title: "A definition isn't replaced before it is used"
languages: [python]
fix: manual
fix_reason: "which definition is meant, and whether to rename or remove the other, needs a person"
stability: preview
ported_from: [ruff/F811]
since: 0.1.0
---

A function, class or import isn't redefined (by another `def`, `class`, import or assignment) before anything reads it.

**Why:** The first definition is dead: usually a copy-paste mistake, or a name clash with an import.

```codelaw
native: py/redefined-while-unused
```

## How to fix

1. Decide which definition is meant.
2. Rename or remove the other one.

## Examples

### Python

#### Don't

```py
def load():
    return 1


def load():
    return 2
```

#### Do

```py
def load():
    return 1


def reload():
    return 2
```

## Differences from the source

A read anywhere between the two definitions counts as a use, including inside a function body Python only runs later.
