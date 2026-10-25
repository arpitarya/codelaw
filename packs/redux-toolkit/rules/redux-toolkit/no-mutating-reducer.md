---
id: redux-toolkit/no-mutating-reducer
title: "A hand-written reducer doesn't change its state in place"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the update has to be rewritten to return a new value"
stability: preview
since: 0.1.0
---

A hand-written reducer — a function `(state, action)` that tests `action.type`, outside
`createSlice`, `createReducer` and `produce` — does not assign to, delete from, or call a mutating
method (`push`, `splice`, `sort`, `Object.assign(state, …)`, …) on its `state`.

**Why:** Redux compares state by reference: a state changed in place is the same object, so the UI doesn't
update, and time-travel debugging shows the wrong history. *Style Guide, Priority A: Do Not Mutate
State.* Inside `createSlice` the state is an Immer draft, and changing it is how updates are written.

```codelaw
native: redux-toolkit/no-mutating-reducer
```

## How to fix

1. Return a new value: `[...state, item]`, `{ ...state, field: value }`, `state.filter(…)`.
2. Better, rewrite the reducer with `createSlice`, where the in-place update is correct.

## Examples

### JavaScript

#### Don't

```js
function todos(state = [], action) {
  switch (action.type) {
    case 'todos/added':
      state.push(action.payload);
      return state;
    default:
      return state;
  }
}
```

#### Do

```js
function todos(state = [], action) {
  switch (action.type) {
    case 'todos/added':
      return [...state, action.payload];
    default:
      return state;
  }
}
```

### JSX

#### Don't

```jsx
function todos(state = [], action) {
  switch (action.type) {
    case 'todos/added':
      state.push(action.payload);
      return state;
    default:
      return state;
  }
}
```

#### Do

```jsx
function todos(state = [], action) {
  switch (action.type) {
    case 'todos/added':
      return [...state, action.payload];
    default:
      return state;
  }
}
```

### TypeScript

#### Don't

```ts
function todos(state = [], action) {
  switch (action.type) {
    case 'todos/added':
      state.push(action.payload);
      return state;
    default:
      return state;
  }
}
```

#### Do

```ts
function todos(state = [], action) {
  switch (action.type) {
    case 'todos/added':
      return [...state, action.payload];
    default:
      return state;
  }
}
```

### TSX

#### Don't

```tsx
function todos(state = [], action) {
  switch (action.type) {
    case 'todos/added':
      state.push(action.payload);
      return state;
    default:
      return state;
  }
}
```

#### Do

```tsx
function todos(state = [], action) {
  switch (action.type) {
    case 'todos/added':
      return [...state, action.payload];
    default:
      return state;
  }
}
```
