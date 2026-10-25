---
id: tanstack-query/no-mutating-set-query-data
title: "A `setQueryData` updater does not change the cached data in place"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the update has to be rewritten to build a new value"
stability: preview
since: 0.3.0
---

The updater function passed to `setQueryData` or `setQueriesData` does not assign to, `delete` from,
increment, `Object.assign` onto or call an in-place array method (`push`, `splice`, `sort`, …) on its
argument, the data in the cache.

**Why:** The cache and every component reading it still hold the same object, so nothing re-renders and the
data a later update starts from is already changed. *Updates from Mutation Responses: "Updates via
`setQueryData` must be performed in an immutable way. **DO NOT** attempt to write directly to the
cache by mutating data (that you retrieved from the cache) in place."*

```codelaw
native: tanstack-query/no-mutating-set-query-data
```

## How to fix

1. Return a new value built from the old one: `{ ...old, title }`, `[...old, item]`,
   `old.filter(…)`, `old.map(…)`.
2. For deep updates, build the copy with Immer's `produce(old, (draft) => { … })`, which is
   where an in-place change is correct.

## Examples

### JavaScript

#### Don't

```js
queryClient.setQueryData(['posts', { id }], (oldData) => {
  if (oldData) {
    oldData.title = 'my new post title';
  }
  return oldData;
});
```

#### Do

```js
queryClient.setQueryData(['posts', { id }], (oldData) =>
  oldData ? { ...oldData, title: 'my new post title' } : oldData,
);
```

### JSX

#### Don't

```jsx
queryClient.setQueryData(['posts', { id }], (oldData) => {
  if (oldData) {
    oldData.title = 'my new post title';
  }
  return oldData;
});
```

#### Do

```jsx
queryClient.setQueryData(['posts', { id }], (oldData) =>
  oldData ? { ...oldData, title: 'my new post title' } : oldData,
);
```

### TypeScript

#### Don't

```ts
queryClient.setQueryData(['posts', { id }], (oldData) => {
  if (oldData) {
    oldData.title = 'my new post title';
  }
  return oldData;
});
```

#### Do

```ts
queryClient.setQueryData(['posts', { id }], (oldData) =>
  oldData ? { ...oldData, title: 'my new post title' } : oldData,
);
```

### TSX

#### Don't

```tsx
queryClient.setQueryData(['posts', { id }], (oldData) => {
  if (oldData) {
    oldData.title = 'my new post title';
  }
  return oldData;
});
```

#### Do

```tsx
queryClient.setQueryData(['posts', { id }], (oldData) =>
  oldData ? { ...oldData, title: 'my new post title' } : oldData,
);
```

## Differences from the source

The source is TanStack Query v5's Updates from Mutation Responses guide, not a lint rule. codelaw reads
an updater written in place and the changes made through its parameter's own name: a value taken out of
it first (`const page = old.pages[0]`), or an item a callback receives (`old.map((p) => { p.seen = true })`),
is not followed. An updater that reassigns its parameter (`old = { ...old }`) is not checked. The method
is recognised by its name, on any object.
