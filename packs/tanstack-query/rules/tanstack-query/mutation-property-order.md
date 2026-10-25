---
id: tanstack-query/mutation-property-order
title: "A mutation's `onMutate` comes before `onError` and `onSettled`"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: ["@tanstack/eslint-plugin-query/mutation-property-order"]
since: 0.1.0
---

In the options of `useMutation`, `onMutate` is written before `onError` and `onSettled`.

**Why:** TypeScript infers the context `onMutate` returns from the order the options are written: written
after them, `onError` and `onSettled` get an `unknown` context.

```codelaw
native: tanstack-query/mutation-property-order
```

## How to fix

1. Move `onMutate` above `onError` and `onSettled`. The fix is offered only when every moved option is
   a function or a name.

## Examples

### JavaScript

#### Don't

```js
useMutation({
  mutationFn: addTodo,
  onError: (error, todo, context) => rollback(context),
  onMutate: (todo) => snapshot(todo),
});
```

#### Do

```js
useMutation({
  mutationFn: addTodo,
  onMutate: (todo) => snapshot(todo),
  onError: (error, todo, context) => rollback(context),
});
```

### JSX

#### Don't

```jsx
useMutation({
  mutationFn: addTodo,
  onError: (error, todo, context) => rollback(context),
  onMutate: (todo) => snapshot(todo),
});
```

#### Do

```jsx
useMutation({
  mutationFn: addTodo,
  onMutate: (todo) => snapshot(todo),
  onError: (error, todo, context) => rollback(context),
});
```

### TypeScript

#### Don't

```ts
useMutation({
  mutationFn: addTodo,
  onError: (error, todo, context) => rollback(context),
  onMutate: (todo) => snapshot(todo),
});
```

#### Do

```ts
useMutation({
  mutationFn: addTodo,
  onMutate: (todo) => snapshot(todo),
  onError: (error, todo, context) => rollback(context),
});
```

### TSX

#### Don't

```tsx
useMutation({
  mutationFn: addTodo,
  onError: (error, todo, context) => rollback(context),
  onMutate: (todo) => snapshot(todo),
});
```

#### Do

```tsx
useMutation({
  mutationFn: addTodo,
  onMutate: (todo) => snapshot(todo),
  onError: (error, todo, context) => rollback(context),
});
```

## Differences from the source

As `tanstack-query/infinite-query-property-order`: the reordering is offered only when every moved
option's value is a function, a method or a name.
