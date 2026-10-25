---
id: redux-toolkit/prefer-create-slice
title: "Reducers are written with createSlice"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the reducer, its action types and its action creators are rewritten together"
stability: preview
since: 0.1.0
---

A Redux reducer isn't written by hand as a `switch` on `action.type` (or a chain of comparisons with
it); it is written with `createSlice`. A reducer handed to React's `useReducer` is component state,
and is left alone.

**Why:** `createSlice` writes the action types and action creators, and its reducers update an Immer draft,
so the immutable-update mistakes can't happen. *Style Guide, Priority B: Use Redux Toolkit for
Writing Redux Logic; Use Immer for Writing Immutable Updates.*

```codelaw
native: redux-toolkit/prefer-create-slice
```

## How to fix

1. Move the initial state and each `case` into `createSlice({ name, initialState, reducers })`, one
   case reducer per action.
2. Export the generated action creators (`slice.actions`) and the reducer (`slice.reducer`), and
   replace the hand-written action types and creators with them.

## Examples

### JavaScript

#### Don't

```js
function counter(state = 0, action) {
  switch (action.type) {
    case 'counter/incremented':
      return state + 1;
    default:
      return state;
  }
}
```

#### Do

```js
const counterSlice = createSlice({
  name: 'counter',
  initialState: 0,
  reducers: {
    incremented: (state) => state + 1,
  },
});
```

### JSX

#### Don't

```jsx
function counter(state = 0, action) {
  switch (action.type) {
    case 'counter/incremented':
      return state + 1;
    default:
      return state;
  }
}
```

#### Do

```jsx
const counterSlice = createSlice({
  name: 'counter',
  initialState: 0,
  reducers: {
    incremented: (state) => state + 1,
  },
});
```

### TypeScript

#### Don't

```ts
function counter(state = 0, action) {
  switch (action.type) {
    case 'counter/incremented':
      return state + 1;
    default:
      return state;
  }
}
```

#### Do

```ts
const counterSlice = createSlice({
  name: 'counter',
  initialState: 0,
  reducers: {
    incremented: (state) => state + 1,
  },
});
```

### TSX

#### Don't

```tsx
function counter(state = 0, action) {
  switch (action.type) {
    case 'counter/incremented':
      return state + 1;
    default:
      return state;
  }
}
```

#### Do

```tsx
const counterSlice = createSlice({
  name: 'counter',
  initialState: 0,
  reducers: {
    incremented: (state) => state + 1,
  },
});
```
