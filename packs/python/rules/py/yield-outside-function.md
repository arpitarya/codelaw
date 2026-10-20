---
id: py/yield-outside-function
title: "`yield` and `await` only inside a function"
languages: [python]
fix: manual
fix_reason: "the code has to be moved into a function, which changes its structure"
stability: preview
ported_from: [ruff/F704]
since: 0.1.0
---

`yield`, `yield from` and `await` appear only inside a function, never at module or class level.

**Why:** Outside a function they are a `SyntaxError`.

```codelaw
native: py/yield-outside-function
```

## How to fix

1. Move the code into a function (a generator, or an `async def` for `await`).

## Examples

### Python

#### Don't

```py
class Numbers:
    yield 1
```

#### Do

```py
def numbers():
    yield 1
```

## Differences from the source

Ruff allows a top-level `await` in notebooks; codelaw doesn't lint notebooks.
