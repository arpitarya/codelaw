---
id: tanstack-query/infinite-query-property-order
title: "An infinite query's `queryFn` comes before its page-param callbacks"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: ["@tanstack/eslint-plugin-query/infinite-query-property-order"]
since: 0.1.0
---

In the options of `useInfiniteQuery`, `useSuspenseInfiniteQuery` and `infiniteQueryOptions`,
`queryFn` is written before `getPreviousPageParam` and `getNextPageParam`.

**Why:** TypeScript infers the page type from the options in the order they are written: written first, the
page-param callbacks get `unknown` pages instead of `queryFn`'s result.

```codelaw
native: tanstack-query/infinite-query-property-order
```

## How to fix

1. Move `queryFn` above `getPreviousPageParam` and `getNextPageParam`. The fix is offered only when
   every moved option is a function or a name, so the order nothing else runs in is kept.

## Examples

### JavaScript

#### Don't

```js
useInfiniteQuery({
  queryKey: ['projects'],
  getNextPageParam: (lastPage) => lastPage.next,
  queryFn: ({ pageParam }) => fetchProjects(pageParam),
  initialPageParam: 0,
});
```

#### Do

```js
useInfiniteQuery({
  queryKey: ['projects'],
  queryFn: ({ pageParam }) => fetchProjects(pageParam),
  getNextPageParam: (lastPage) => lastPage.next,
  initialPageParam: 0,
});
```

### JSX

#### Don't

```jsx
useInfiniteQuery({
  queryKey: ['projects'],
  getNextPageParam: (lastPage) => lastPage.next,
  queryFn: ({ pageParam }) => fetchProjects(pageParam),
  initialPageParam: 0,
});
```

#### Do

```jsx
useInfiniteQuery({
  queryKey: ['projects'],
  queryFn: ({ pageParam }) => fetchProjects(pageParam),
  getNextPageParam: (lastPage) => lastPage.next,
  initialPageParam: 0,
});
```

### TypeScript

#### Don't

```ts
useInfiniteQuery({
  queryKey: ['projects'],
  getNextPageParam: (lastPage) => lastPage.next,
  queryFn: ({ pageParam }) => fetchProjects(pageParam),
  initialPageParam: 0,
});
```

#### Do

```ts
useInfiniteQuery({
  queryKey: ['projects'],
  queryFn: ({ pageParam }) => fetchProjects(pageParam),
  getNextPageParam: (lastPage) => lastPage.next,
  initialPageParam: 0,
});
```

### TSX

#### Don't

```tsx
useInfiniteQuery({
  queryKey: ['projects'],
  getNextPageParam: (lastPage) => lastPage.next,
  queryFn: ({ pageParam }) => fetchProjects(pageParam),
  initialPageParam: 0,
});
```

#### Do

```tsx
useInfiniteQuery({
  queryKey: ['projects'],
  queryFn: ({ pageParam }) => fetchProjects(pageParam),
  getNextPageParam: (lastPage) => lastPage.next,
  initialPageParam: 0,
});
```

## Differences from the source

Upstream always offers the reordering. codelaw offers it only when every moved option's value is a
function, a method or a name, since moving an expression changes when it runs; otherwise the finding
has no edit.
