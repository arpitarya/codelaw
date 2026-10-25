---
id: redux-toolkit/no-sequential-dispatch
title: "One event is one action, not a run of dispatches"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the actions have to be merged into one that the reducers all respond to"
stability: preview
options:
  max: { type: "integer", default: 2, description: "how many `dispatch` statements in a row are accepted" }
since: 0.1.0
---

A block doesn't dispatch more than `max` (2) actions in a row, statement after statement.

**Why:** Each dispatch re-renders the subscribers and leaves the state halfway between the first action and
the last, a state no reducer meant. *Style Guide, Priority B: Avoid Dispatching Many Actions
Sequentially; Model Actions as Events, Not Setters.*

```codelaw
native: redux-toolkit/no-sequential-dispatch
```

## How to fix

1. Dispatch one action that describes what happened (`orderPlaced`), with what the reducers need in
   its payload.
2. Let each slice that cares respond to it in `extraReducers`.

## Examples

### JavaScript

#### Don't

```js
function placeOrder(dispatch) {
  dispatch(setPizzas(1));
  dispatch(setCokes(1));
  dispatch(setTotal(12));
}
```

#### Do

```js
function placeOrder(dispatch) {
  dispatch(orderPlaced({ pizza: 1, coke: 1 }));
}
```

### JSX

#### Don't

```jsx
function placeOrder(dispatch) {
  dispatch(setPizzas(1));
  dispatch(setCokes(1));
  dispatch(setTotal(12));
}
```

#### Do

```jsx
function placeOrder(dispatch) {
  dispatch(orderPlaced({ pizza: 1, coke: 1 }));
}
```

### TypeScript

#### Don't

```ts
function placeOrder(dispatch) {
  dispatch(setPizzas(1));
  dispatch(setCokes(1));
  dispatch(setTotal(12));
}
```

#### Do

```ts
function placeOrder(dispatch) {
  dispatch(orderPlaced({ pizza: 1, coke: 1 }));
}
```

### TSX

#### Don't

```tsx
function placeOrder(dispatch) {
  dispatch(setPizzas(1));
  dispatch(setCokes(1));
  dispatch(setTotal(12));
}
```

#### Do

```tsx
function placeOrder(dispatch) {
  dispatch(orderPlaced({ pizza: 1, coke: 1 }));
}
```
