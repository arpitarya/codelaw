---
id: redux-toolkit/no-side-effects-in-reducer
title: "A reducer has no side effects"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the side effect has to move to a thunk, a listener or a `prepare` callback"
stability: preview
since: 0.1.0
---

A reducer — a `createSlice` case reducer, a builder's `addCase`/`addMatcher`/`addDefaultCase`
callback, or a hand-written `(state, action)` reducer — doesn't fetch, wait, start timers, read the
clock or a random source, touch storage, or dispatch.

**Why:** A reducer must give the same next state for the same state and action, so that replaying actions
reproduces the state. *Style Guide, Priority A: Reducers Must Not Have Side Effects.*

```codelaw
native: redux-toolkit/no-side-effects-in-reducer
```

## How to fix

1. Make random IDs and timestamps in the action's `prepare` callback, so they arrive in the payload.
2. Move fetching, timers and storage into a thunk (`createAsyncThunk`) or a listener.

## Examples

### JavaScript

#### Don't

```js
const todosSlice = createSlice({
  name: 'todos',
  initialState: [],
  reducers: {
    todoAdded(state, action) {
      state.push({ id: nanoid(), text: action.payload });
    },
  },
});
```

#### Do

```js
const todosSlice = createSlice({
  name: 'todos',
  initialState: [],
  reducers: {
    todoAdded: {
      reducer(state, action) {
        state.push(action.payload);
      },
      prepare(text) {
        return { payload: { id: nanoid(), text } };
      },
    },
  },
});
```

### JSX

#### Don't

```jsx
const todosSlice = createSlice({
  name: 'todos',
  initialState: [],
  reducers: {
    todoAdded(state, action) {
      state.push({ id: nanoid(), text: action.payload });
    },
  },
});
```

#### Do

```jsx
const todosSlice = createSlice({
  name: 'todos',
  initialState: [],
  reducers: {
    todoAdded: {
      reducer(state, action) {
        state.push(action.payload);
      },
      prepare(text) {
        return { payload: { id: nanoid(), text } };
      },
    },
  },
});
```

### TypeScript

#### Don't

```ts
const todosSlice = createSlice({
  name: 'todos',
  initialState: [],
  reducers: {
    todoAdded(state, action) {
      state.push({ id: nanoid(), text: action.payload });
    },
  },
});
```

#### Do

```ts
const todosSlice = createSlice({
  name: 'todos',
  initialState: [],
  reducers: {
    todoAdded: {
      reducer(state, action) {
        state.push(action.payload);
      },
      prepare(text) {
        return { payload: { id: nanoid(), text } };
      },
    },
  },
});
```

### TSX

#### Don't

```tsx
const todosSlice = createSlice({
  name: 'todos',
  initialState: [],
  reducers: {
    todoAdded(state, action) {
      state.push({ id: nanoid(), text: action.payload });
    },
  },
});
```

#### Do

```tsx
const todosSlice = createSlice({
  name: 'todos',
  initialState: [],
  reducers: {
    todoAdded: {
      reducer(state, action) {
        state.push(action.payload);
      },
      prepare(text) {
        return { payload: { id: nanoid(), text } };
      },
    },
  },
});
```
