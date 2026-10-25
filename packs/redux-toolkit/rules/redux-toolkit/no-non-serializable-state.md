---
id: redux-toolkit/no-non-serializable-state
title: "State holds only serializable values"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the data has to be stored in another shape, and the code reading it changed"
stability: preview
since: 0.1.0
---

A slice's `initialState` (in `createSlice` or `createReducer`, followed through a `const`) and the
values a reducer assigns into its state are plain objects, arrays and primitives: no `Map`, `Set`,
`Date`, `Promise`, class instance, `Symbol`, function or regular expression.

**Why:** The DevTools, persistence and time-travel all serialize the state; a value that doesn't survive
`JSON.stringify` breaks them, and Redux Toolkit's serializability check warns at run time. *Style
Guide, Priority A: Do Not Put Non-Serializable Values in State or Actions.*

```codelaw
native: redux-toolkit/no-non-serializable-state
```

## How to fix

1. Store a `Map` or `Set` as an object keyed by ID, or as an array.
2. Store a `Date` as its ISO string or its timestamp, and make the `Date` where it is displayed.
3. Keep functions and class instances outside the store; store an ID that finds them.

## Examples

### JavaScript

#### Don't

```js
const usersSlice = createSlice({
  name: 'users',
  initialState: { byId: new Map() },
  reducers: {},
});
```

#### Do

```js
const usersSlice = createSlice({
  name: 'users',
  initialState: { byId: {} },
  reducers: {},
});
```

### JSX

#### Don't

```jsx
const usersSlice = createSlice({
  name: 'users',
  initialState: { byId: new Map() },
  reducers: {},
});
```

#### Do

```jsx
const usersSlice = createSlice({
  name: 'users',
  initialState: { byId: {} },
  reducers: {},
});
```

### TypeScript

#### Don't

```ts
const usersSlice = createSlice({
  name: 'users',
  initialState: { byId: new Map() },
  reducers: {},
});
```

#### Do

```ts
const usersSlice = createSlice({
  name: 'users',
  initialState: { byId: {} },
  reducers: {},
});
```

### TSX

#### Don't

```tsx
const usersSlice = createSlice({
  name: 'users',
  initialState: { byId: new Map() },
  reducers: {},
});
```

#### Do

```tsx
const usersSlice = createSlice({
  name: 'users',
  initialState: { byId: {} },
  reducers: {},
});
```
