---
id: tanstack-query/exhaustive-deps
title: "A query key names every value its query function reads"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "a new key entry changes which cache entry the query reads and writes"
stability: preview
options:
  allowlist: { type: "object", default: {}, description: "`{ variables: [<name>…], types: [<type name>…] }`: values to leave out of the key, by name or by their declared type" }
ported_from: ["@tanstack/eslint-plugin-query/exhaustive-deps"]
since: 0.1.0
---

An object with a `queryKey` and a `queryFn` lists in its key every prop, state value and local of the
component that the query function reads.

**Why:** TanStack Query caches by key. A value the function reads but the key leaves out makes two different
queries share one cache entry, so a change to it shows stale data and refetches nothing.

```codelaw
native: tanstack-query/exhaustive-deps
```

## Examples

### JavaScript

#### Don't

```js
import { useQuery } from '@tanstack/react-query';

function useTodo(id) {
  return useQuery({ queryKey: ['todo'], queryFn: () => fetchTodo(id) });
}
```

#### Do

```js
import { useQuery } from '@tanstack/react-query';

function useTodo(id) {
  return useQuery({ queryKey: ['todo', id], queryFn: () => fetchTodo(id) });
}
```

### JSX

#### Don't

```jsx
import { useQuery } from '@tanstack/react-query';

function useTodo(id) {
  return useQuery({ queryKey: ['todo'], queryFn: () => fetchTodo(id) });
}
```

#### Do

```jsx
import { useQuery } from '@tanstack/react-query';

function useTodo(id) {
  return useQuery({ queryKey: ['todo', id], queryFn: () => fetchTodo(id) });
}
```

### TypeScript

#### Don't

```ts
import { useQuery } from '@tanstack/react-query';

function useTodo(id) {
  return useQuery({ queryKey: ['todo'], queryFn: () => fetchTodo(id) });
}
```

#### Do

```ts
import { useQuery } from '@tanstack/react-query';

function useTodo(id) {
  return useQuery({ queryKey: ['todo', id], queryFn: () => fetchTodo(id) });
}
```

### TSX

#### Don't

```tsx
import { useQuery } from '@tanstack/react-query';

function useTodo(id) {
  return useQuery({ queryKey: ['todo'], queryFn: () => fetchTodo(id) });
}
```

#### Do

```tsx
import { useQuery } from '@tanstack/react-query';

function useTodo(id) {
  return useQuery({ queryKey: ['todo', id], queryFn: () => fetchTodo(id) });
}
```

## Differences from the source

A value counts when it is declared in the outermost function around the options object, outside the
query function; upstream counts only that function's own top-level names. A member path stops at the
first subscript (`a[i].b` needs `a`). A key held in a `const` is followed to its array, and the
suggested edit lands there. `allowlist.types` matches the type names written in the value's own
annotation; codelaw has no type checker to resolve an inferred type.
