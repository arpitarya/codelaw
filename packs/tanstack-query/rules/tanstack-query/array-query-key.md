---
id: tanstack-query/array-query-key
title: "A query key is an array"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "wrapping the key in an array changes its hash, so a key matched elsewhere in its old form stops matching"
stability: preview
since: 0.3.0
---

The `queryKey` of a query's options or of query filters is an array, not a string, a number or an
object written in its place.

**Why:** TanStack Query hashes and matches keys as arrays, and its types require one. *Query Keys: "Query keys
have to be an Array at the top level".*

```codelaw
native: tanstack-query/array-query-key
```

## How to fix

1. Wrap the key in an array: `queryKey: ['todos']`.

## Examples

### JavaScript

#### Don't

```js
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: 'todos', queryFn: fetchTodos });
```

#### Do

```js
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
```

### JSX

#### Don't

```jsx
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: 'todos', queryFn: fetchTodos });
```

#### Do

```jsx
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
```

### TypeScript

#### Don't

```ts
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: 'todos', queryFn: fetchTodos });
```

#### Do

```ts
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
```

### TSX

#### Don't

```tsx
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: 'todos', queryFn: fetchTodos });
```

#### Do

```tsx
import { useQuery } from '@tanstack/react-query';

useQuery({ queryKey: ['todos'], queryFn: fetchTodos });
```

## Differences from the source

The source is TanStack Query v5's Query Keys guide, not a lint rule. Only a key written in place as a
string, template string, number, boolean or object literal is reported; a key passed by name is not
followed. The object must be one TanStack Query reads: next to a `queryFn`, an argument of a function
imported from a `@tanstack/*-query` package or of a QueryClient method, or an entry of `queries`.
