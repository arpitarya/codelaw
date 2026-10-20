---
id: py/useless-expression
title: "No expression statement that does nothing"
languages: [python]
fix: manual
fix_reason: "what the expression was meant for needs a person"
stability: preview
ported_from: [ruff/B018]
since: 0.1.0
---

A statement isn't a bare name, number, attribute access or container literal whose value is thrown away.

**Why:** It has no effect; usually something was meant to be returned, assigned or called.

```codelaw
native: py/useless-expression
```

## How to fix

1. Return, assign or call the value as intended, or remove the line.

## Examples

### Python

#### Don't

```py
def name(user):
    user.name
    return user
```

#### Do

```py
def name(user):
    return user.name
```

## Differences from the source

Attribute access is reported only on plain `a.b.c` chains; Ruff also reports `f().attr`.
