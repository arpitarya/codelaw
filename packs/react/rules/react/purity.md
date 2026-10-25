---
id: react/purity
title: "Render doesn't call Math.random(), Date.now() or other impure functions"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "where the value should come from instead — state, an effect, an event — depends on what it is for"
stability: preview
ported_from: [eslint-plugin-react-hooks/purity, "@eslint-react/purity"]
since: 0.3.0
---

A component or hook doesn't call `Math.random()`, `Date.now()`, `performance.now()`,
`crypto.randomUUID()` or `crypto.getRandomValues()`, or make `new Date()` with no argument, in its
own body. Calls in callbacks — effects, event handlers, a `useState` initializer — are fine, and so
is a local binding that shadows the global (`Math` as a parameter).

**Why:** React may render a component any number of times and expects the same output for the same props and state; a value that changes on every call makes the output change on every render and breaks hydration.

```codelaw
native: react/purity
```

## How to fix

1. For a value made once, compute it in a `useState` initializer: `useState(() => Date.now())`.
2. For an id, use `useId()`.
3. For a value that should change, set it in an effect or an event handler.

## Examples

### JavaScript

#### Don't

```js
function useRandomKey() {
  return Math.random().toString(36);
}
```

#### Do

```js
function useRandomKey() {
  const [key] = useState(() => Math.random().toString(36));
  return key;
}
```

### JSX

#### Don't

```jsx
function Clock() {
  const now = Date.now();
  return <p>{now}</p>;
}
```

#### Do

```jsx
function Clock() {
  const [now] = useState(() => Date.now());
  return <p>{now}</p>;
}
```

### TypeScript

#### Don't

```ts
function useToday(): string {
  return new Date().toDateString();
}
```

#### Do

```ts
function useToday(): string {
  const [today] = useState(() => new Date().toDateString());
  return today;
}
```

### TSX

#### Don't

```tsx
function Field({ label }: { label: string }) {
  const id = crypto.randomUUID();
  return <label htmlFor={id}>{label}</label>;
}
```

#### Do

```tsx
function Field({ label }: { label: string }) {
  const id = useId();
  return <label htmlFor={id}>{label}</label>;
}
```

## Differences from the source

- eslint-plugin-react-hooks 7 knows `Math.random`, `Date.now` and `performance.now`; codelaw adds `crypto.randomUUID`, `crypto.getRandomValues` and `new Date()` from @eslint-react's list, and leaves out the rest of that list (DOM, storage, network and timer calls), which reads as side effects more than impurity and is better reported by a rule of its own.
- Upstream follows a global through an alias (`const { random } = Math`); codelaw reports only the direct `Math.random()` form.
- A call in a function the component calls during render is not followed.
