---
id: py/function-uses-loop-variable
title: "Closures don't capture a loop variable by name"
languages: [python]
fix: manual
fix_reason: "binding the value now (a default argument, `functools.partial`) changes the function's signature"
stability: preview
ported_from: [ruff/B023]
since: 0.1.0
---

A function or lambda defined in a loop doesn't read a variable the loop rebinds, unless it is called or consumed straight away.

**Why:** Python closures look names up when they run, so every closure sees the loop's last value.

```codelaw
native: py/function-uses-loop-variable
```

## How to fix

1. Bind the current value with a default argument (`lambda name=name: …`) or `functools.partial`.

## Examples

### Python

#### Don't

```py
callbacks = []
for name in ["a", "b"]:
    callbacks.append(lambda: print(name))
```

#### Do

```py
callbacks = []
for name in ["a", "b"]:
    callbacks.append(lambda name=name: print(name))
```
