---
id: py/percent-format-unused-named-argument
title: "Every key given to `%` is used"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/F504]
since: 0.1.0
---

A dict literal formatted into a `%` string has no keys the string never names.

**Why:** An unused key is usually a misspelled placeholder.

```codelaw
native: py/percent-format-unused-named-argument
```

## Examples

### Python

#### Don't

```py
message = "%(name)s" % {"name": "Ada", "age": 36}
```

#### Do

```py
message = "%(name)s" % {"name": "Ada"}
```

## Differences from the source

The fix is offered only when the removed values are plain names or literals, so no call is dropped.
