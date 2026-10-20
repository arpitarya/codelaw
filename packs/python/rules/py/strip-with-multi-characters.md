---
id: py/strip-with-multi-characters
title: "`.strip()` isn't given a word"
languages: [python]
fix: manual
fix_reason: "whether a character set or a suffix was meant needs a person"
stability: preview
ported_from: [ruff/B005]
since: 0.1.0
---

`.strip()`, `.lstrip()` and `.rstrip()` aren't passed a multi-character string with repeated characters.

**Why:** They remove any of the given characters, not a prefix or suffix; `"note.txt".rstrip(".txt")` gives `"no"`.

```codelaw
native: py/strip-with-multi-characters
```

## How to fix

1. Use `.removeprefix()` / `.removesuffix()` to remove a fixed string.
2. Keep `.strip()` only for a set of characters, written without repeats.

## Examples

### Python

#### Don't

```py
name = filename.rstrip(".txt")
```

#### Do

```py
name = filename.removesuffix(".txt")
```
