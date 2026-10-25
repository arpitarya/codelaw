---
id: tanstack-query/no-unsupported-suspense-options
title: "A suspense query takes no `enabled`, `placeholderData` or `throwOnError`"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "removing `enabled` makes the query always run, and removing `placeholderData` shows the fallback during an update"
stability: preview
since: 0.3.0
---

`useSuspenseQuery`, `useSuspenseInfiniteQuery` and the entries of `useSuspenseQueries` are not given
`enabled`, `placeholderData` or `throwOnError`.

**Why:** The suspense hooks don't read them: the query always runs, always suspends, and throws when it has no
data. *Suspense: "you therefore can't conditionally enable / disable the Query"; "`placeholderData`
also doesn't exist for this Query"; "you can't change `throwOnError`".*

```codelaw
native: tanstack-query/no-unsupported-suspense-options
```

## How to fix

1. `enabled`: render the component only when the query should run, or use `useQuery`.
2. `placeholderData`: wrap the update that changes the query key in `startTransition`.
3. `throwOnError`: throw `error` yourself when `!isFetching`, to send every error to the boundary.

## Examples

### JavaScript

#### Don't

```js
import { useSuspenseQuery } from '@tanstack/react-query';

function Todo({ id }) {
  const { data } = useSuspenseQuery({
    queryKey: ['todo', id],
    queryFn: () => fetchTodo(id),
    enabled: !!id,
  });
  return data;
}
```

#### Do

```js
import { useSuspenseQuery } from '@tanstack/react-query';

function Todo({ id }) {
  const { data } = useSuspenseQuery({
    queryKey: ['todo', id],
    queryFn: () => fetchTodo(id),
  });
  return data;
}
```

### JSX

#### Don't

```jsx
import { useSuspenseQuery } from '@tanstack/react-query';

function Todo({ id }) {
  const { data } = useSuspenseQuery({
    queryKey: ['todo', id],
    queryFn: () => fetchTodo(id),
    enabled: !!id,
  });
  return data;
}
```

#### Do

```jsx
import { useSuspenseQuery } from '@tanstack/react-query';

function Todo({ id }) {
  const { data } = useSuspenseQuery({
    queryKey: ['todo', id],
    queryFn: () => fetchTodo(id),
  });
  return data;
}
```

### TypeScript

#### Don't

```ts
import { useSuspenseQuery } from '@tanstack/react-query';

function Todo({ id }) {
  const { data } = useSuspenseQuery({
    queryKey: ['todo', id],
    queryFn: () => fetchTodo(id),
    enabled: !!id,
  });
  return data;
}
```

#### Do

```ts
import { useSuspenseQuery } from '@tanstack/react-query';

function Todo({ id }) {
  const { data } = useSuspenseQuery({
    queryKey: ['todo', id],
    queryFn: () => fetchTodo(id),
  });
  return data;
}
```

### TSX

#### Don't

```tsx
import { useSuspenseQuery } from '@tanstack/react-query';

function Todo({ id }) {
  const { data } = useSuspenseQuery({
    queryKey: ['todo', id],
    queryFn: () => fetchTodo(id),
    enabled: !!id,
  });
  return data;
}
```

#### Do

```tsx
import { useSuspenseQuery } from '@tanstack/react-query';

function Todo({ id }) {
  const { data } = useSuspenseQuery({
    queryKey: ['todo', id],
    queryFn: () => fetchTodo(id),
  });
  return data;
}
```

## Differences from the source

The source is TanStack Query v5's Suspense guide, not a lint rule; TypeScript reports these options too,
because v5's types leave them out of the suspense hooks' options. codelaw reports them in JavaScript as
well. Only option objects written in place are read, and the hook must be imported from a
`@tanstack/*-query` package.
