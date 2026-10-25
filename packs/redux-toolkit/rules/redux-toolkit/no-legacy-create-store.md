---
id: redux-toolkit/no-legacy-create-store
title: "The store is made with configureStore"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "`configureStore` takes its options in another shape, and the middleware and enhancers move"
stability: preview
since: 0.1.0
---

`createStore` and `legacy_createStore` are not imported from `redux`; the store is made with Redux
Toolkit's `configureStore`.

**Why:** `configureStore` sets up the thunk middleware, the DevTools and the mutation and serializability
checks that catch the Priority A mistakes; `createStore` sets up none of them. *Style Guide, Priority
B: Use Redux Toolkit for Writing Redux Logic.*

```codelaw
match:
  kind: import_specifier
  has: { field: name, regex: ^(createStore|legacy_createStore)$ }
  inside:
    kind: import_statement
    has: { field: source, regex: "^['\"]redux['\"]$" }
    stopBy: end
message: "Make the store with Redux Toolkit's `configureStore`, which adds the thunk middleware, the DevTools and the development checks"
```

## How to fix

1. Import `configureStore` from `@reduxjs/toolkit`.
2. Replace `createStore(rootReducer, preloadedState, enhancer)` with
   `configureStore({ reducer: rootReducer, preloadedState })`; pass extra middleware through
   `middleware: (getDefault) => getDefault().concat(…)`.

## Examples

### JavaScript

#### Don't

```js
import { createStore } from 'redux';
```

#### Do

```js
import { configureStore } from '@reduxjs/toolkit';
```

### JSX

#### Don't

```jsx
import { createStore } from 'redux';
```

#### Do

```jsx
import { configureStore } from '@reduxjs/toolkit';
```

### TypeScript

#### Don't

```ts
import { createStore } from 'redux';
```

#### Do

```ts
import { configureStore } from '@reduxjs/toolkit';
```

### TSX

#### Don't

```tsx
import { createStore } from 'redux';
```

#### Do

```tsx
import { configureStore } from '@reduxjs/toolkit';
```
