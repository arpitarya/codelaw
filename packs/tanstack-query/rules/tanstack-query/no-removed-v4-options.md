---
id: tanstack-query/no-removed-v4-options
title: "A query takes no option TanStack Query v5 removed"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "each removed option is replaced by different code: an effect, `placeholderData`, `structuralSharing` or a hook argument"
stability: preview
since: 0.3.0
---

A query's options — in a query hook, `queryOptions`, a QueryClient query method or
`defaultOptions.queries` — do not include `onSuccess`, `onError`, `onSettled`, `keepPreviousData`,
`isDataEqual` or `context`. Mutations keep their callbacks.

**Why:** v5 removed these options and ignores them, so the code they hold never runs. *Migrating to TanStack
Query v5: "`onSuccess`, `onError` and `onSettled` have been removed from Queries. They haven't been
touched for Mutations"; "Removed `keepPreviousData` in favor of `placeholderData` identity function";
"The `isDataEqual` option has been removed"; "Removed custom `context` prop in favor of custom
`queryClient` instance".*

```codelaw
native: tanstack-query/no-removed-v4-options
```

## How to fix

1. `onSuccess` / `onError` / `onSettled`: react to `data` or `error` where the query is rendered, move the
   work into the `queryFn`, or use the `QueryCache`'s global callbacks.
2. `keepPreviousData: true`: write `placeholderData: keepPreviousData`, importing `keepPreviousData`
   from TanStack Query, and read `isPlaceholderData` instead of `isPreviousData`.
3. `isDataEqual`: pass `structuralSharing: (old, next) => (equal(old, next) ? old : replaceEqualDeep(old, next))`.
4. `context`: pass the `QueryClient` as the hook's second argument.

## Examples

### JavaScript

#### Don't

```js
import { useQuery } from '@tanstack/react-query';

function Projects({ page }) {
  const { data } = useQuery({
    queryKey: ['projects', page],
    queryFn: () => fetchProjects(page),
    keepPreviousData: true,
  });
  return data;
}
```

#### Do

```js
import { useQuery, keepPreviousData } from '@tanstack/react-query';

function Projects({ page }) {
  const { data } = useQuery({
    queryKey: ['projects', page],
    queryFn: () => fetchProjects(page),
    placeholderData: keepPreviousData,
  });
  return data;
}
```

### JSX

#### Don't

```jsx
import { useQuery } from '@tanstack/react-query';

function Projects({ page }) {
  const { data } = useQuery({
    queryKey: ['projects', page],
    queryFn: () => fetchProjects(page),
    keepPreviousData: true,
  });
  return data;
}
```

#### Do

```jsx
import { useQuery, keepPreviousData } from '@tanstack/react-query';

function Projects({ page }) {
  const { data } = useQuery({
    queryKey: ['projects', page],
    queryFn: () => fetchProjects(page),
    placeholderData: keepPreviousData,
  });
  return data;
}
```

### TypeScript

#### Don't

```ts
import { useQuery } from '@tanstack/react-query';

function Projects({ page }) {
  const { data } = useQuery({
    queryKey: ['projects', page],
    queryFn: () => fetchProjects(page),
    keepPreviousData: true,
  });
  return data;
}
```

#### Do

```ts
import { useQuery, keepPreviousData } from '@tanstack/react-query';

function Projects({ page }) {
  const { data } = useQuery({
    queryKey: ['projects', page],
    queryFn: () => fetchProjects(page),
    placeholderData: keepPreviousData,
  });
  return data;
}
```

### TSX

#### Don't

```tsx
import { useQuery } from '@tanstack/react-query';

function Projects({ page }) {
  const { data } = useQuery({
    queryKey: ['projects', page],
    queryFn: () => fetchProjects(page),
    keepPreviousData: true,
  });
  return data;
}
```

#### Do

```tsx
import { useQuery, keepPreviousData } from '@tanstack/react-query';

function Projects({ page }) {
  const { data } = useQuery({
    queryKey: ['projects', page],
    queryFn: () => fetchProjects(page),
    placeholderData: keepPreviousData,
  });
  return data;
}
```

## Differences from the source

The source is TanStack Query v5's migration guide, not a lint rule. Only option objects written in place
are read, and a hook is recognised only when it is imported from a `@tanstack/*-query` package. The
other v4 removals the guide lists — `refetchPage`, the custom logger, the `remove` method on a result,
`dehydrateMutations` — are not options of the objects this rule reads, and are not checked. The guide
also says the `suspense: boolean` flag was removed, but v5's `@tanstack/query-core` still declares
`suspense` on a query observer's options and `useBaseQuery` still reads it (the suspense hooks set it), so
the option is not ignored and is not reported.
