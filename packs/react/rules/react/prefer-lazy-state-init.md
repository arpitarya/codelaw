---
id: react/prefer-lazy-state-init
title: "useState's initial value is not computed on every render"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "wrapping changes behaviour when the value is itself a function, and moves the call's side effects to the first render only"
stability: preview
ported_from: ["@eslint-react/use-state"]
since: 0.3.0
---

`useState`'s argument doesn't call a function or construct an object on every render:
`useState(load())` and `useState(new Map())` compute a value that React uses only on the first
render. Pass a function instead — `useState(() => load())` — and React calls it once. Calls of
hooks, of `Boolean`, `String` and `Number`, of a method (`JSON.parse(…)`), and calls inside
`use(…)` or inside a function are not reported.

**Why:** The argument is evaluated on every render even though React ignores it after the first, so an expensive call runs for nothing each time.

```codelaw
native: react/prefer-lazy-state-init
```

## Examples

### JavaScript

#### Don't

```js
function useTodos() {
  const [todos, setTodos] = useState(loadTodos());
  return [todos, setTodos];
}
```

#### Do

```js
function useTodos() {
  const [todos, setTodos] = useState(() => loadTodos());
  return [todos, setTodos];
}
```

### JSX

#### Don't

```jsx
function Cart() {
  const [items] = useState(new Map());
  return <p>{items.size}</p>;
}
```

#### Do

```jsx
function Cart() {
  const [items] = useState(() => new Map());
  return <p>{items.size}</p>;
}
```

### TypeScript

#### Don't

```ts
function useFilters() {
  const [filters, setFilters] = useState({ tags: readTags() });
  return [filters, setFilters] as const;
}
```

#### Do

```ts
function useFilters() {
  const [filters, setFilters] = useState(() => ({ tags: readTags() }));
  return [filters, setFilters] as const;
}
```

### TSX

#### Don't

```tsx
function Editor() {
  const [doc] = useState(createDocument());
  return <article>{doc.title}</article>;
}
```

#### Do

```tsx
function Editor() {
  const [doc] = useState(() => createDocument());
  return <article>{doc.title}</article>;
}
```

## Differences from the source

- This is the lazy-initialisation part of @eslint-react's `use-state`; its other two checks — that the call is destructured, and that the setter is named `set` + the value's name — are naming conventions, not ported here.
- Upstream reports each call in the argument; codelaw reports the argument once, naming the first call, and offers one fix for it.
