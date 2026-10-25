---
id: react/no-set-state-in-effect
title: "An effect doesn't set state synchronously"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether to derive the value, lift the state or subscribe is a design choice"
stability: preview
ported_from: [eslint-plugin-react-hooks/set-state-in-effect, "@eslint-react/set-state-in-effect"]
since: 0.3.0
---

The callback of `useEffect`, `useLayoutEffect` or `useInsertionEffect` doesn't call a `useState`
setter directly — conditionally or not. A setter called in a callback the effect sets up (a
subscription, a timer, a promise's `then`) is fine, and so is one given a value read from a ref
(`setX(ref.current)`).

**Why:** An effect runs after React has committed a render; setting state there commits a second render at once, so every change costs two renders, and the first one shows stale values.

```codelaw
native: react/no-set-state-in-effect
```

## How to fix

1. If the value can be computed from props or state, compute it during render and drop the state and the effect.
2. If it depends on a change of a prop, handle the event that changed the prop instead.
3. If it comes from an outside source, subscribe in the effect and set the state in the subscription's callback, or use `useSyncExternalStore`.

## Examples

### JavaScript

#### Don't

```js
function useFullName(first, last) {
  const [full, setFull] = useState('');
  useEffect(() => {
    setFull(first + ' ' + last);
  }, [first, last]);
  return full;
}
```

#### Do

```js
function useWidth() {
  const [width, setWidth] = useState(0);
  useEffect(() => {
    const onResize = () => setWidth(window.innerWidth);
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);
  return width;
}
```

### JSX

#### Don't

```jsx
function Name({ first, last }) {
  const [full, setFull] = useState('');
  useEffect(() => {
    setFull(first + ' ' + last);
  }, [first, last]);
  return <p>{full}</p>;
}
```

#### Do

```jsx
function Name({ first, last }) {
  const full = first + ' ' + last;
  return <p>{full}</p>;
}
```

### TypeScript

#### Don't

```ts
function useFullName(first: string, last: string) {
  const [full, setFull] = useState('');
  useEffect(() => {
    setFull(first + ' ' + last);
  }, [first, last]);
  return full;
}
```

#### Do

```ts
function useWidth(): number {
  const [width, setWidth] = useState(0);
  useEffect(() => {
    const onResize = () => setWidth(window.innerWidth);
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);
  return width;
}
```

### TSX

#### Don't

```tsx
function Name({ first, last }: { first: string; last: string }) {
  const [full, setFull] = useState('');
  useEffect(() => {
    setFull(first + ' ' + last);
  }, [first, last]);
  return <p>{full}</p>;
}
```

#### Do

```tsx
function Name({ first, last }: { first: string; last: string }) {
  const full = first + ' ' + last;
  return <p>{full}</p>;
}
```

## Differences from the source

- Upstream follows the setter through local functions the effect calls; codelaw reports only a direct call of the setter in the effect's own body.
- Upstream exempts a value derived from a ref by data flow; codelaw exempts a call whose arguments mention `.current`.
