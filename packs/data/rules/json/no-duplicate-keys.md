---
id: json/no-duplicate-keys
title: "No key twice in one JSON object"
languages: [json]
fix: manual
fix_reason: "which of the two values was meant can't be known from the file"
stability: preview
since: 0.1.0
---

Each key appears once in a JSON object. Keys are compared by value, so `"a"` and `"\u0061"` are the same key.

**Why:** RFC 8259 leaves duplicate keys undefined: most parsers keep the last value and silently drop the first, others keep the first or refuse the file.

```codelaw
native: json/no-duplicate-keys
```

## How to fix

1. Decide which value the key should have.
2. Delete the other entry, or move it under a key of its own.

## Examples

### JSON

#### Don't

```json
{
  "name": "app",
  "version": "1.0.0",
  "name": "web-app"
}
```

#### Do

```json
{
  "name": "web-app",
  "version": "1.0.0"
}
```
