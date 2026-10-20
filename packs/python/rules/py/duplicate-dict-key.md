---
id: py/duplicate-dict-key
title: "A dict literal doesn't repeat a literal key"
languages: [python]
fix: manual
fix_reason: "which value should win can't be known from the code"
stability: preview
ported_from: [ruff/F601]
since: 0.1.0
---

A dict literal names each literal key (`"a"`, `1`, `True`) once.

**Why:** Only the last value is kept, so the earlier one is silently lost.

```codelaw
native: py/duplicate-dict-key
```

## How to fix

1. Decide which value the key should have.
2. Remove the other entry, or give it its own key.

## Examples

### Python

#### Don't

```py
colors = {"red": 1, "red": 2}
```

#### Do

```py
colors = {"red": 1, "blue": 2}
```
