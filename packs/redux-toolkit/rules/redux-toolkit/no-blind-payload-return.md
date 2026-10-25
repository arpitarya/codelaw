---
id: redux-toolkit/no-blind-payload-return
title: "A reducer builds its state, not the action's payload"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the reducer has to name the fields it takes from the payload"
stability: preview
since: 0.1.0
---

A reducer doesn't return `action.payload` as the new state, return an object spreading
`...action.payload`, or `Object.assign` the payload onto an Immer draft.

**Why:** Whoever dispatches the action then decides the shape of the state, and a payload with a missing or
extra field changes it without the reducer knowing. *Style Guide, Priority B: Reducers Should Own the
State Shape.* The guide allows a spread for a form that edits every field; `redux-toolkit/recommended`
warns rather than errs.

```codelaw
native: redux-toolkit/no-blind-payload-return
```

## How to fix

1. Copy the fields the state owns from the payload by name: `state.name = action.payload.name`.
2. If replacing the whole state is the point (a cache reset), write that as its own case with a
   descriptive action name, and suppress the finding there.

## Examples

### JavaScript

#### Don't

```js
const userSlice = createSlice({
  name: 'user',
  initialState: { name: '', email: '' },
  reducers: {
    userUpdated(state, action) {
      return action.payload;
    },
  },
});
```

#### Do

```js
const userSlice = createSlice({
  name: 'user',
  initialState: { name: '', email: '' },
  reducers: {
    userUpdated(state, action) {
      state.name = action.payload.name;
      state.email = action.payload.email;
    },
  },
});
```

### JSX

#### Don't

```jsx
const userSlice = createSlice({
  name: 'user',
  initialState: { name: '', email: '' },
  reducers: {
    userUpdated(state, action) {
      return action.payload;
    },
  },
});
```

#### Do

```jsx
const userSlice = createSlice({
  name: 'user',
  initialState: { name: '', email: '' },
  reducers: {
    userUpdated(state, action) {
      state.name = action.payload.name;
      state.email = action.payload.email;
    },
  },
});
```

### TypeScript

#### Don't

```ts
const userSlice = createSlice({
  name: 'user',
  initialState: { name: '', email: '' },
  reducers: {
    userUpdated(state, action) {
      return action.payload;
    },
  },
});
```

#### Do

```ts
const userSlice = createSlice({
  name: 'user',
  initialState: { name: '', email: '' },
  reducers: {
    userUpdated(state, action) {
      state.name = action.payload.name;
      state.email = action.payload.email;
    },
  },
});
```

### TSX

#### Don't

```tsx
const userSlice = createSlice({
  name: 'user',
  initialState: { name: '', email: '' },
  reducers: {
    userUpdated(state, action) {
      return action.payload;
    },
  },
});
```

#### Do

```tsx
const userSlice = createSlice({
  name: 'user',
  initialState: { name: '', email: '' },
  reducers: {
    userUpdated(state, action) {
      state.name = action.payload.name;
      state.email = action.payload.email;
    },
  },
});
```
