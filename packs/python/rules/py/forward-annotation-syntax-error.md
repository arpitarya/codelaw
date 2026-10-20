---
id: py/forward-annotation-syntax-error
title: "A string annotation is valid Python"
languages: [python]
fix: manual
fix_reason: "the intended type can't be inferred from broken text"
stability: preview
ported_from: [ruff/F722]
since: 0.1.0
---

A type annotation written as a string parses as a Python expression.

**Why:** Type checkers and `typing.get_type_hints()` fail on an annotation they can't parse.

```codelaw
native: py/forward-annotation-syntax-error
```

## How to fix

1. Correct the annotation so it parses.

## Examples

### Python

#### Don't

```py
def total(values: "list[int") -> int:
    return sum(values)
```

#### Do

```py
def total(values: "list[int]") -> int:
    return sum(values)
```

## Differences from the source

Strings inside `Literal[...]` and `Annotated[...]` are not treated as annotations.
