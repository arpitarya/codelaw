---
id: react/globals
title: "Render doesn't change module-level or global values"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "where the change belongs — state, a ref, an effect, an event — depends on what the value is for"
stability: preview
ported_from: [eslint-plugin-react-hooks/globals, "@eslint-react/globals"]
since: 0.3.0
---

A component or hook doesn't assign, increment or set a property of a value declared outside
it — a module-level variable or a global such as `window` — in its own body. Changes made in
effects, event handlers and other callbacks are fine.

**Why:** React may render a component any number of times, in any order, and throw renders away; a module-level value changed during render changes a different number of times than the component appears to render, and leaks between components and requests.

```codelaw
native: react/globals
```

## How to fix

1. If the value belongs to the component, keep it in state or a ref.
2. If it must change outside React, change it in an effect or an event handler.

## Examples

### JavaScript

#### Don't

```js
let renders = 0;

function useRenderCount() {
  renders += 1;
  return renders;
}
```

#### Do

```js
function useRenderCount() {
  const renders = useRef(0);
  useEffect(() => {
    renders.current += 1;
  });
  return renders;
}
```

### JSX

#### Don't

```jsx
function Page({ title }) {
  window.pageTitle = title;
  return <h1>{title}</h1>;
}
```

#### Do

```jsx
function Page({ title }) {
  useEffect(() => {
    window.pageTitle = title;
  }, [title]);
  return <h1>{title}</h1>;
}
```

### TypeScript

#### Don't

```ts
const seen: Record<string, boolean> = {};

function useSeen(id: string) {
  seen[id] = true;
  return Object.keys(seen).length;
}
```

#### Do

```ts
const seen: Record<string, boolean> = {};

function useSeen(id: string) {
  useEffect(() => {
    seen[id] = true;
  }, [id]);
  return id in seen;
}
```

### TSX

#### Don't

```tsx
function Page({ title }: { title: string }) {
  window.pageTitle = title;
  return <h1>{title}</h1>;
}
```

#### Do

```tsx
function Page({ title }: { title: string }) {
  useEffect(() => {
    window.pageTitle = title;
  }, [title]);
  return <h1>{title}</h1>;
}
```

## Differences from the source

- Only assignments, `+=`-style updates and `++` / `--` are reported; upstream also reports mutating method calls on a global array (`list.push(x)`), which needs to know the value is an array.
- Changes in a function the component calls during render are not followed.
