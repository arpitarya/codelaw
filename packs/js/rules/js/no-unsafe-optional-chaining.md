---
id: js/no-unsafe-optional-chaining
title: "No ?. chain used where undefined throws"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the right fallback value depends on the code"
stability: preview
ported_from: [eslint/no-unsafe-optional-chaining]
since: 0.1.0
---

A `?.` chain, which can evaluate to `undefined`, is not called through parentheses, constructed,
spread into an array or call, destructured, iterated with `for … of`, used on the right of `in` or
`instanceof`, or extended.

**Why:** In those places `undefined` throws a TypeError, which defeats the point of `?.`.

```codelaw
native: js/no-unsafe-optional-chaining
```

## How to fix

1. Give the chain a fallback with `??`.
2. Or move the `?.` so that it covers the operation that can fail.

## Examples

### JavaScript

#### Don't

```js
const { name } = user?.profile;
```

#### Do

```js
const { name } = user?.profile ?? {};
```

### JSX

#### Don't

```jsx
const { name } = user?.profile;
```

#### Do

```jsx
const { name } = user?.profile ?? {};
```

### TypeScript

#### Don't

```ts
const { name } = user?.profile;
```

#### Do

```ts
const { name } = user?.profile ?? {};
```

### TSX

#### Don't

```tsx
const { name } = user?.profile;
```

#### Do

```tsx
const { name } = user?.profile ?? {};
```
