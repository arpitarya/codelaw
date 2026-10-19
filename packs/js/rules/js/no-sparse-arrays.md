---
id: js/no-sparse-arrays
title: "No holes in array literals"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether a value is missing or a comma is extra needs a reader"
stability: preview
ported_from: [eslint/no-sparse-arrays]
since: 0.1.0
---

An array literal has no empty slots such as `[1, , 2]`.

**Why:** A hole is usually a typo, and holes behave differently from `undefined` in many array
methods.

```codelaw
native: js/no-sparse-arrays
```

## How to fix

1. Put the missing value in the slot, or remove the extra comma.
2. If an empty slot is really meant, write `undefined` explicitly.

## Examples

### JavaScript

#### Don't

```js
const colors = ["red", , "blue"];
```

#### Do

```js
const colors = ["red", "green", "blue"];
```

### JSX

#### Don't

```jsx
const colors = ["red", , "blue"];
```

#### Do

```jsx
const colors = ["red", "green", "blue"];
```

### TypeScript

#### Don't

```ts
const colors = ["red", , "blue"];
```

#### Do

```ts
const colors = ["red", "green", "blue"];
```

### TSX

#### Don't

```tsx
const colors = ["red", , "blue"];
```

#### Do

```tsx
const colors = ["red", "green", "blue"];
```
