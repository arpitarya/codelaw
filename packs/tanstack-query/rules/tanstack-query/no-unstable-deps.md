---
id: tanstack-query/no-unstable-deps
title: "A query hook's result is not a React hook dependency"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the fields to list depend on what the effect, callback or memo reads"
stability: preview
ported_from: ["@tanstack/eslint-plugin-query/no-unstable-deps"]
since: 0.1.0
---

The whole result of `useQuery`, `useMutation` and the other query hooks — or of a custom hook in the
same file that returns one — is not listed in the dependency array of `useEffect`, `useCallback` or
`useMemo`.

**Why:** The result object is new on every render, so a dependency on it makes the effect run, and the
callback or memo change, on every render.

```codelaw
native: tanstack-query/no-unstable-deps
```

## How to fix

1. Destructure the fields the hook reads: `const { mutate } = useMutation(…)`.
2. List those fields in the dependency array instead of the whole result.

## Examples

### JavaScript

#### Don't

```js
import { useMutation } from '@tanstack/react-query';
import { useCallback } from 'react';

function useSave() {
  const mutation = useMutation({ mutationFn: save });
  return useCallback(() => mutation.mutate(), [mutation]);
}
```

#### Do

```js
import { useMutation } from '@tanstack/react-query';
import { useCallback } from 'react';

function useSave() {
  const { mutate } = useMutation({ mutationFn: save });
  return useCallback(() => mutate(), [mutate]);
}
```

### JSX

#### Don't

```jsx
import { useMutation } from '@tanstack/react-query';
import { useCallback } from 'react';

function useSave() {
  const mutation = useMutation({ mutationFn: save });
  return useCallback(() => mutation.mutate(), [mutation]);
}
```

#### Do

```jsx
import { useMutation } from '@tanstack/react-query';
import { useCallback } from 'react';

function useSave() {
  const { mutate } = useMutation({ mutationFn: save });
  return useCallback(() => mutate(), [mutate]);
}
```

### TypeScript

#### Don't

```ts
import { useMutation } from '@tanstack/react-query';
import { useCallback } from 'react';

function useSave() {
  const mutation = useMutation({ mutationFn: save });
  return useCallback(() => mutation.mutate(), [mutation]);
}
```

#### Do

```ts
import { useMutation } from '@tanstack/react-query';
import { useCallback } from 'react';

function useSave() {
  const { mutate } = useMutation({ mutationFn: save });
  return useCallback(() => mutate(), [mutate]);
}
```

### TSX

#### Don't

```tsx
import { useMutation } from '@tanstack/react-query';
import { useCallback } from 'react';

function useSave() {
  const mutation = useMutation({ mutationFn: save });
  return useCallback(() => mutation.mutate(), [mutation]);
}
```

#### Do

```tsx
import { useMutation } from '@tanstack/react-query';
import { useCallback } from 'react';

function useSave() {
  const { mutate } = useMutation({ mutationFn: save });
  return useCallback(() => mutate(), [mutate]);
}
```

## Differences from the source

A custom hook is followed one level, as upstream does: a hook in the same file whose body is, or
returns, one direct query hook call.
