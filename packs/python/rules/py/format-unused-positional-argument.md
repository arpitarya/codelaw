---
id: py/format-unused-positional-argument
title: "Every positional argument to `.format` is used"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/F523]
since: 0.1.0
---

A `.format()` call has no positional arguments the string never refers to.

**Why:** An extra argument is usually a missing `{}` field.

```codelaw
native: py/format-unused-positional-argument
```

## Examples

### Python

#### Don't

```py
message = "{}".format("Ada", 36)
```

#### Do

```py
message = "{}".format("Ada")
```

## Differences from the source

The fix removes unused arguments only from the end of the list, and only plain names or literals.
