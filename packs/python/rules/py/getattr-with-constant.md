---
id: py/getattr-with-constant
title: "No `getattr` with a constant name"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/B009]
since: 0.1.0
---

`getattr(obj, "name")` with a constant, valid attribute name is written `obj.name`.

**Why:** `getattr` with a constant name is no safer than attribute access, only harder to read.

```codelaw
native: py/getattr-with-constant
```

## Examples

### Python

#### Don't

```py
name = getattr(user, "name")
```

#### Do

```py
name = user.name
```
