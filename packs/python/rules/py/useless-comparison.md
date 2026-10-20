---
id: py/useless-comparison
title: "No comparison whose result is thrown away"
languages: [python]
fix: manual
fix_reason: "whether an assignment or an assertion was meant needs a person"
stability: preview
ported_from: [ruff/B015]
since: 0.1.0
---

A comparison isn't written as a statement on its own.

**Why:** Its result is discarded; usually `=` or `assert` was meant.

```codelaw
native: py/useless-comparison
```

## How to fix

1. Use `=` if an assignment was meant, or `assert` if a check was.
2. Otherwise remove the line.

## Examples

### Python

#### Don't

```py
def reset(state):
    state.count == 0
```

#### Do

```py
def reset(state):
    state.count = 0
```
