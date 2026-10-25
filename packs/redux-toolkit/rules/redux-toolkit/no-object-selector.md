---
id: redux-toolkit/no-object-selector
title: "useSelector selects one value per call"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the one call is split into one per value, and the destructuring goes"
stability: preview
since: 0.1.0
---

A selector passed alone to `useSelector` (or a typed `use…Selector` hook) doesn't return a new object
literal.

**Why:** `useSelector` re-renders when the selected value changes by reference, and a new object is a new
reference after every dispatched action, so the component re-renders on all of them. *Style Guide,
Priority B: Call useSelector Multiple Times in Function Components.* An equality function such as
`shallowEqual` as the second argument avoids it, and is left alone.

```codelaw
native: redux-toolkit/no-object-selector
```

## How to fix

1. Call `useSelector` once per value: `const todos = useSelector((state) => state.todos)`.
2. Or pass `shallowEqual` as the second argument when one object is really wanted.

## Examples

### JavaScript

#### Don't

```js
function TodoList() {
  const { todos, filter } = useSelector((state) => ({ todos: state.todos, filter: state.filter }));
  return [todos, filter];
}
```

#### Do

```js
function TodoList() {
  const todos = useSelector((state) => state.todos);
  const filter = useSelector((state) => state.filter);
  return [todos, filter];
}
```

### JSX

#### Don't

```jsx
function TodoList() {
  const { todos, filter } = useSelector((state) => ({ todos: state.todos, filter: state.filter }));
  return [todos, filter];
}
```

#### Do

```jsx
function TodoList() {
  const todos = useSelector((state) => state.todos);
  const filter = useSelector((state) => state.filter);
  return [todos, filter];
}
```

### TypeScript

#### Don't

```ts
function TodoList() {
  const { todos, filter } = useSelector((state) => ({ todos: state.todos, filter: state.filter }));
  return [todos, filter];
}
```

#### Do

```ts
function TodoList() {
  const todos = useSelector((state) => state.todos);
  const filter = useSelector((state) => state.filter);
  return [todos, filter];
}
```

### TSX

#### Don't

```tsx
function TodoList() {
  const { todos, filter } = useSelector((state) => ({ todos: state.todos, filter: state.filter }));
  return [todos, filter];
}
```

#### Do

```tsx
function TodoList() {
  const todos = useSelector((state) => state.todos);
  const filter = useSelector((state) => state.filter);
  return [todos, filter];
}
```
