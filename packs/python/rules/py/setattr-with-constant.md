---
id: py/setattr-with-constant
title: "No `setattr` with a constant name"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/B010]
since: 0.1.0
---

`setattr(obj, "name", value)` as a statement, with a constant, valid attribute name, is written `obj.name = value`.

**Why:** `setattr` with a constant name is no safer than an assignment, only harder to read.

```codelaw
native: py/setattr-with-constant
```

## Examples

### Python

#### Don't

```py
setattr(user, "name", "Ada")
```

#### Do

```py
user.name = "Ada"
```
