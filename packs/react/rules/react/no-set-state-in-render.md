---
id: react/no-set-state-in-render
title: "State is not set on every render"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what the state should be instead — derived during render, or set in an event — is a design choice"
stability: preview
ported_from: [eslint-plugin-react-hooks/set-state-in-render, "@eslint-react/set-state-in-render"]
since: 0.3.0
---

A component or hook doesn't call a `useState` setter unconditionally in its own body: not inside a
condition, a loop, a callback or after an early `return`. The setter is recognised by its
declaration — the second element of `const [value, setValue] = useState(…)` in the same function.

**Why:** Setting state during render schedules another render, which sets the state again: the component renders forever, or React stops it with an error.

```codelaw
native: react/no-set-state-in-render
```

## How to fix

1. If the value can be computed from props or other state, compute it during render instead of storing it.
2. If the state must follow a prop, set it only when the prop changes: keep the previous prop in state and compare, inside an `if`.
3. Otherwise set it where the change happens: an event handler, or an effect's subscription callback.

## Examples

### JavaScript

#### Don't

```js
function useCounter(start) {
  const [count, setCount] = useState(0);
  setCount(start);
  return count;
}
```

#### Do

```js
function useCounter(start) {
  const [count, setCount] = useState(start);
  return [count, setCount];
}
```

### JSX

#### Don't

```jsx
function Counter({ start }) {
  const [count, setCount] = useState(0);
  setCount(start);
  return <p>{count}</p>;
}
```

#### Do

```jsx
function Counter({ start }) {
  const [count, setCount] = useState(start);
  return <button onClick={() => setCount(count + 1)}>{count}</button>;
}
```

### TypeScript

#### Don't

```ts
function useCounter(start: number) {
  const [count, setCount] = useState(0);
  setCount(start);
  return count;
}
```

#### Do

```ts
function useCounter(start: number) {
  const [count, setCount] = useState(start);
  return [count, setCount] as const;
}
```

### TSX

#### Don't

```tsx
function Counter({ start }: { start: number }) {
  const [count, setCount] = useState(0);
  setCount(start);
  return <p>{count}</p>;
}
```

#### Do

```tsx
function Counter({ start }: { start: number }) {
  const [count, setCount] = useState(start);
  return <button onClick={() => setCount(count + 1)}>{count}</button>;
}
```

## Differences from the source

- The React Compiler follows a setter through aliases and helper functions called during render; codelaw sees only a direct call of the setter's own name in the body of the function that declared it.
- A call inside `useMemo`'s callback (which upstream also reports) is not reported: it is in a nested function.
