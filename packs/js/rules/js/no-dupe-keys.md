---
id: js/no-dupe-keys
title: "No two keys in an object literal are the same"
languages: [javascript, jsx]
fix: manual
fix_reason: "which value should win can't be known from the code"
stability: preview
ported_from: [eslint/no-dupe-keys]
since: 0.1.0
---

An object literal names each key once; `a`, `'a'` and `["a"]` are the same key, as are `1` and
`0x1`.

**Why:** The later value silently replaces the earlier one.

```codelaw
native: js/no-dupe-keys
```

## How to fix

1. Decide which value the key should have.
2. Remove the other entry, or give it its own key.

## Examples

### JavaScript

#### Don't

```js
const user = {
  name: "Ada",
  name: "Grace",
};
```

#### Do

```js
const user = {
  name: "Ada",
  nickname: "Grace",
};
```

### JSX

#### Don't

```jsx
const user = {
  name: "Ada",
  name: "Grace",
};
```

#### Do

```jsx
const user = {
  name: "Ada",
  nickname: "Grace",
};
```
