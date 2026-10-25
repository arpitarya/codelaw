---
id: react/no-unnecessary-use-prefix
title: "A use… function calls a hook"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "renaming the function means renaming every caller, in other files too"
stability: preview
ported_from: ["@eslint-react/no-unnecessary-use-prefix"]
since: 0.3.0
---

A function named `use…` calls at least one hook somewhere in its body. Not reported: an empty
function, `use` and `useMDXComponents`, a function whose comments name a hook call (`// useX()`
later), and functions passed to a test mock (`vi.mock`, `jest.mock`, `mock.module`).

**Why:** The `use` prefix tells React's tools — and readers — that the Rules of Hooks apply; on a plain function it forbids calling it in a condition or a loop for no reason.

```codelaw
native: react/no-unnecessary-use-prefix
```

## How to fix

1. Rename the function without the `use` prefix (`useFormat` → `format`), and rename its callers.
2. If it is meant to become a hook, leave a comment naming the hook it will call.

## Examples

### JavaScript

#### Don't

```js
function useFormatDate(date) {
  return date.toISOString().slice(0, 10);
}
```

#### Do

```js
function formatDate(date) {
  return date.toISOString().slice(0, 10);
}
```

### JSX

#### Don't

```jsx
const useGreeting = (name) => <p>Hello, {name}</p>;
```

#### Do

```jsx
const greeting = (name) => <p>Hello, {name}</p>;
```

### TypeScript

#### Don't

```ts
function useTotal(items: number[]): number {
  return items.reduce((a, b) => a + b, 0);
}
```

#### Do

```ts
function useTotal(items: number[]): number {
  return useMemo(() => items.reduce((a, b) => a + b, 0), [items]);
}
```

### TSX

#### Don't

```tsx
const useGreeting = (name: string) => <p>Hello, {name}</p>;
```

#### Do

```tsx
const greeting = (name: string) => <p>Hello, {name}</p>;
```

## Differences from the source

- A hook call anywhere in the body counts, nested callbacks included; upstream counts the hook calls its collector attributes to the function.
