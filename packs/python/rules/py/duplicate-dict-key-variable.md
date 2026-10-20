---
id: py/duplicate-dict-key-variable
title: "A dict literal doesn't repeat a variable key"
languages: [python]
fix: manual
fix_reason: "which value should win can't be known from the code"
stability: preview
ported_from: [ruff/F602]
since: 0.1.0
---

A dict literal uses the same variable as a key at most once.

**Why:** Both entries have the same key at run time, so only the last value is kept.

```codelaw
native: py/duplicate-dict-key-variable
```

## How to fix

1. Decide which entry is meant.
2. Remove the other one, or use a different key.

## Examples

### Python

#### Don't

```py
scores = {player: 1, player: 2}
```

#### Do

```py
scores = {player: 1, rival: 2}
```
