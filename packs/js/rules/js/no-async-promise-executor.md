---
id: js/no-async-promise-executor
title: "A Promise executor is not an async function"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the rewrite depends on what the executor does"
stability: preview
ported_from: [eslint/no-async-promise-executor]
since: 0.1.0
---

The function passed to `new Promise(...)` is not `async`.

**Why:** An error thrown in an async executor doesn't reject the promise, and an executor that needs
`await` doesn't need the `Promise` wrapper.

```codelaw
match:
  any:
    - pattern: new Promise(async ($$$P) => $$$B)
    - pattern: new Promise(async function ($$$P) { $$$B })
    - pattern: new Promise(async $P => $$$B)
message: "An async Promise executor loses its errors; await inside a plain async function instead"
```

## How to fix

1. If the executor only awaits and resolves, use the awaited promise directly.
2. Otherwise make the executor a plain function and handle errors with `reject`.

## Examples

### JavaScript

#### Don't

```js
const data = new Promise(async (resolve) => {
  resolve(await load());
});
```

#### Do

```js
const data = load();
```

### JSX

#### Don't

```jsx
const data = new Promise(async (resolve) => {
  resolve(await load());
});
```

#### Do

```jsx
const data = load();
```

### TypeScript

#### Don't

```ts
const data = new Promise(async (resolve) => {
  resolve(await load());
});
```

#### Do

```ts
const data = load();
```

### TSX

#### Don't

```tsx
const data = new Promise(async (resolve) => {
  resolve(await load());
});
```

#### Do

```tsx
const data = load();
```
