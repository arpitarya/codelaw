---
id: tanstack-query/no-renamed-v4-options
title: "An option uses its TanStack Query v5 name"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
since: 0.3.0
---

The options of a query or a mutation — in a hook, a `queryOptions` / `mutationOptions` helper, a
QueryClient method or `new QueryClient({ defaultOptions })` — use v5's names: `gcTime`, not `cacheTime`,
and `throwOnError`, not `useErrorBoundary`.

**Why:** v5 reads only the new names, so an option under its v4 name does nothing. *Migrating to TanStack
Query v5: "Rename `cacheTime` to `gcTime`" and "The `useErrorBoundary` option has been renamed to
`throwOnError`".*

```codelaw
native: tanstack-query/no-renamed-v4-options
```

## How to fix

1. Rename the key. The fix is not offered when the object already has the new name too.

## Examples

### JavaScript

#### Don't

```js
import { useQuery } from '@tanstack/react-query';

useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodos,
  cacheTime: 10 * 60 * 1000,
  useErrorBoundary: true,
});
```

#### Do

```js
import { useQuery } from '@tanstack/react-query';

useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodos,
  gcTime: 10 * 60 * 1000,
  throwOnError: true,
});
```

### JSX

#### Don't

```jsx
import { useQuery } from '@tanstack/react-query';

useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodos,
  cacheTime: 10 * 60 * 1000,
  useErrorBoundary: true,
});
```

#### Do

```jsx
import { useQuery } from '@tanstack/react-query';

useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodos,
  gcTime: 10 * 60 * 1000,
  throwOnError: true,
});
```

### TypeScript

#### Don't

```ts
import { useQuery } from '@tanstack/react-query';

useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodos,
  cacheTime: 10 * 60 * 1000,
  useErrorBoundary: true,
});
```

#### Do

```ts
import { useQuery } from '@tanstack/react-query';

useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodos,
  gcTime: 10 * 60 * 1000,
  throwOnError: true,
});
```

### TSX

#### Don't

```tsx
import { useQuery } from '@tanstack/react-query';

useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodos,
  cacheTime: 10 * 60 * 1000,
  useErrorBoundary: true,
});
```

#### Do

```tsx
import { useQuery } from '@tanstack/react-query';

useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodos,
  gcTime: 10 * 60 * 1000,
  throwOnError: true,
});
```

## Differences from the source

The source is TanStack Query v5's migration guide, not a lint rule. codelaw reads option objects written
in place: an options object built elsewhere and passed by name is not checked. A hook or helper is
recognised only when it is imported from a `@tanstack/*-query` package; a QueryClient method
(`prefetchQuery`, `fetchQuery`, `ensureQueryData`, the infinite variants, `setQueryDefaults`,
`setMutationDefaults`) by its name alone.
