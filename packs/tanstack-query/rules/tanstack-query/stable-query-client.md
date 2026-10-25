---
id: tanstack-query/stable-query-client
title: "The QueryClient is made once, not on every render"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "the fix calls `React.useState` (or the imported `useState`), which must be in scope"
stability: preview
ported_from: ["@tanstack/eslint-plugin-query/stable-query-client"]
since: 0.1.0
---

`new QueryClient()` from `@tanstack/react-query` is not assigned inside a component or hook; it is made
at module level, or once in `useState`.

**Why:** A client made during render is a new, empty cache on every render: every query refetches and nothing
is shared. An async Server Component runs once per request and may make one.

```codelaw
native: tanstack-query/stable-query-client
```

## Examples

### JavaScript

#### Don't

```js
import { QueryClient } from '@tanstack/react-query';

function useClient() {
  const queryClient = new QueryClient();
  return queryClient;
}
```

#### Do

```js
import { QueryClient } from '@tanstack/react-query';

function useClient() {
  const [queryClient] = React.useState(() => new QueryClient());
  return queryClient;
}
```

### JSX

#### Don't

```jsx
import { QueryClient } from '@tanstack/react-query';

function useClient() {
  const queryClient = new QueryClient();
  return queryClient;
}
```

#### Do

```jsx
import { QueryClient } from '@tanstack/react-query';

function useClient() {
  const [queryClient] = React.useState(() => new QueryClient());
  return queryClient;
}
```

### TypeScript

#### Don't

```ts
import { QueryClient } from '@tanstack/react-query';

function useClient() {
  const queryClient = new QueryClient();
  return queryClient;
}
```

#### Do

```ts
import { QueryClient } from '@tanstack/react-query';

function useClient() {
  const [queryClient] = React.useState(() => new QueryClient());
  return queryClient;
}
```

### TSX

#### Don't

```tsx
import { QueryClient } from '@tanstack/react-query';

function useClient() {
  const queryClient = new QueryClient();
  return queryClient;
}
```

#### Do

```tsx
import { QueryClient } from '@tanstack/react-query';

function useClient() {
  const [queryClient] = React.useState(() => new QueryClient());
  return queryClient;
}
```

## Differences from the source

A component or hook is recognised by its own name or by the variable it initialises
(`const App = () => …`); upstream reads only a function's own name. The fix uses `useState` when the
file imports it from `react`, else `React.useState`, as upstream does.
