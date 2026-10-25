---
id: tanstack-query/object-syntax
title: "A TanStack Query call takes one options object"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "the positional arguments are moved into a new object, which changes the call's shape"
stability: preview
since: 0.3.0
---

`useQuery`, `useInfiniteQuery`, `useMutation`, `useIsFetching`, `useIsMutating` and the QueryClient
methods that take a query key or filters (`invalidateQueries`, `refetchQueries`, `fetchQuery`,
`prefetchQuery`, …) are called with one options object, not with v4's positional `(key, fn, options)`.

**Why:** v5 removed the positional overloads: it reads the first argument as the options object, so a key passed
there is not a query. *Migrating to TanStack Query v5: "Supports a single signature, one object … now we
only support the object format."*

```codelaw
native: tanstack-query/object-syntax
```

## How to fix

1. Move the arguments into one object: `useQuery(key, fn, options)` becomes
   `useQuery({ queryKey: key, queryFn: fn, ...options })`. The fix is offered when the arguments' roles
   can be read off the call: an options object written in place, or a function written in place.

## Examples

### JavaScript

#### Don't

```js
import { useQuery } from '@tanstack/react-query';

useQuery(['todos'], fetchTodos, { staleTime: 5000 });
```

#### Do

```js
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: ['todos'], queryFn: fetchTodos, staleTime: 5000 });
```

### JSX

#### Don't

```jsx
import { useQuery } from '@tanstack/react-query';

useQuery(['todos'], fetchTodos, { staleTime: 5000 });
```

#### Do

```jsx
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: ['todos'], queryFn: fetchTodos, staleTime: 5000 });
```

### TypeScript

#### Don't

```ts
import { useQuery } from '@tanstack/react-query';

useQuery(['todos'], fetchTodos, { staleTime: 5000 });
```

#### Do

```ts
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: ['todos'], queryFn: fetchTodos, staleTime: 5000 });
```

### TSX

#### Don't

```tsx
import { useQuery } from '@tanstack/react-query';

useQuery(['todos'], fetchTodos, { staleTime: 5000 });
```

#### Do

```tsx
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: ['todos'], queryFn: fetchTodos, staleTime: 5000 });
```

## Differences from the source

The source is TanStack Query v5's migration guide, and the guide's codemod (`remove-overloads`) does the
same rewrite. codelaw reports a call whose first argument is a key written in place (an array or a
string; a string is wrapped in an array) or, for `useMutation`, a function; a key passed by name is not
checked, since v5's options object can be passed by name too. `queryCache.find` and `findAll` are left
out: their names are too common to read as TanStack Query's. The v4 plugin's
`prefer-query-object-syntax` asked for the same object form while v4 still allowed both.
