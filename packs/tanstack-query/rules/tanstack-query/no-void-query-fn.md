---
id: tanstack-query/no-void-query-fn
title: "A query function returns its data"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what the query should return is the data the function fetches, which the author knows"
stability: preview
ported_from: ["@tanstack/eslint-plugin-query/no-void-query-fn"]
since: 0.1.0
---

A `queryFn` returns a value. A function whose body has no `return <value>` and no `throw`, or that
returns `undefined` or `void`, hands TanStack Query nothing to cache.

**Why:** TanStack Query stores what the query function returns, and treats `undefined` as an error: the query
fails, or every consumer reads `undefined`.

```codelaw
native: tanstack-query/no-void-query-fn
```

## How to fix

1. Return the fetched data: `return res.json()`, or `return response.data`.
2. If the function only triggers work, it is a mutation: move it to `useMutation`'s `mutationFn`.

## Examples

### JavaScript

#### Don't

```js
const todosQuery = {
  queryKey: ['todos'],
  queryFn: async () => {
    await fetch('/todos');
  },
};
```

#### Do

```js
const todosQuery = {
  queryKey: ['todos'],
  queryFn: async () => {
    const res = await fetch('/todos');
    return res.json();
  },
};
```

### JSX

#### Don't

```jsx
const todosQuery = {
  queryKey: ['todos'],
  queryFn: async () => {
    await fetch('/todos');
  },
};
```

#### Do

```jsx
const todosQuery = {
  queryKey: ['todos'],
  queryFn: async () => {
    const res = await fetch('/todos');
    return res.json();
  },
};
```

### TypeScript

#### Don't

```ts
const todosQuery = {
  queryKey: ['todos'],
  queryFn: async () => {
    await fetch('/todos');
  },
};
```

#### Do

```ts
const todosQuery = {
  queryKey: ['todos'],
  queryFn: async () => {
    const res = await fetch('/todos');
    return res.json();
  },
};
```

### TSX

#### Don't

```tsx
const todosQuery = {
  queryKey: ['todos'],
  queryFn: async () => {
    await fetch('/todos');
  },
};
```

#### Do

```tsx
const todosQuery = {
  queryKey: ['todos'],
  queryFn: async () => {
    const res = await fetch('/todos');
    return res.json();
  },
};
```

## Differences from the source

Upstream asks the TypeScript type checker for the function's return type. codelaw reads the function
itself: it reports only a `queryFn` written in place whose body cannot return a value. A function
passed by name, or one that returns a value typed as possibly `undefined`, is not checked.
