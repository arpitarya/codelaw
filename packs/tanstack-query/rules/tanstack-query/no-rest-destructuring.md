---
id: tanstack-query/no-rest-destructuring
title: "No rest destructuring of a query result"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which fields the component needs is a choice only its author can make"
stability: preview
ported_from: ["@tanstack/eslint-plugin-query/no-rest-destructuring"]
since: 0.1.0
---

The result of `useQuery`, `useInfiniteQuery`, `useSuspenseQuery` and their `useQueries` forms is not
rest-destructured (`const { data, ...rest } = useQuery(…)`) or spread (`{ ...query }`).

**Why:** A query result tracks which fields are read and re-renders only when one of them changes. A rest
element or a spread reads every field, so the component re-renders on every change to the query.

```codelaw
native: tanstack-query/no-rest-destructuring
```

## How to fix

1. Destructure the fields the component uses by name: `const { data, isPending } = useQuery(…)`.
2. Or keep the result whole (`const query = useQuery(…)`) and read `query.data` where it is used.

## Examples

### JavaScript

#### Don't

```js
import { useQuery } from '@tanstack/react-query';

function useTodos() {
  const { data, ...rest } = useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
  return [data, rest];
}
```

#### Do

```js
import { useQuery } from '@tanstack/react-query';

function useTodos() {
  const { data, isPending } = useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
  return [data, isPending];
}
```

### JSX

#### Don't

```jsx
import { useQuery } from '@tanstack/react-query';

function useTodos() {
  const { data, ...rest } = useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
  return [data, rest];
}
```

#### Do

```jsx
import { useQuery } from '@tanstack/react-query';

function useTodos() {
  const { data, isPending } = useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
  return [data, isPending];
}
```

### TypeScript

#### Don't

```ts
import { useQuery } from '@tanstack/react-query';

function useTodos() {
  const { data, ...rest } = useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
  return [data, rest];
}
```

#### Do

```ts
import { useQuery } from '@tanstack/react-query';

function useTodos() {
  const { data, isPending } = useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
  return [data, isPending];
}
```

### TSX

#### Don't

```tsx
import { useQuery } from '@tanstack/react-query';

function useTodos() {
  const { data, ...rest } = useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
  return [data, rest];
}
```

#### Do

```tsx
import { useQuery } from '@tanstack/react-query';

function useTodos() {
  const { data, isPending } = useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
  return [data, isPending];
}
```

## Differences from the source

Only direct calls of the hooks, imported from a `@tanstack/*-query` package, are followed. Upstream
also finds a query result by its type when typed linting is on; codelaw has no type checker.
