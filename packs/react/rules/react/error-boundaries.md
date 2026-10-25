---
id: react/error-boundaries
title: "Errors in children are caught by an Error Boundary, not try/catch"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "an Error Boundary is a component to add above this one, with its own fallback"
stability: preview
ported_from: [eslint-plugin-react-hooks/error-boundaries, "@eslint-react/error-boundaries"]
since: 0.3.0
---

A component or hook doesn't wrap a `return` of JSX, or a `use(…)` call, in `try`/`catch`. A `try`
around ordinary work — parsing, a computation — is fine.

**Why:** Returning JSX only describes the children; React renders them later, outside the `try`, so what they throw is never caught there. `use` suspends by throwing, and only an Error Boundary (or Suspense) handles what it throws.

```codelaw
native: react/error-boundaries
```

## How to fix

1. Remove the `try`/`catch` around the JSX or the `use(…)` call.
2. Wrap the component's children (or the component, where it is used) in an Error Boundary with a fallback.

## Examples

### JavaScript

#### Don't

```js
function Profile({ user }) {
  try {
    return <Avatar user={user} />;
  } catch {
    return <p>Something went wrong</p>;
  }
}
```

#### Do

```js
function Profile({ user }) {
  return (
    <ErrorBoundary fallback={<p>Something went wrong</p>}>
      <Avatar user={user} />
    </ErrorBoundary>
  );
}
```

### JSX

#### Don't

```jsx
function Profile({ user }) {
  try {
    return <Avatar user={user} />;
  } catch {
    return <p>Something went wrong</p>;
  }
}
```

#### Do

```jsx
function Profile({ user }) {
  return (
    <ErrorBoundary fallback={<p>Something went wrong</p>}>
      <Avatar user={user} />
    </ErrorBoundary>
  );
}
```

### TSX

#### Don't

```tsx
function Comments({ load }: { load: Promise<string[]> }) {
  try {
    const comments = use(load);
    return <ul>{comments.map((c) => <li key={c}>{c}</li>)}</ul>;
  } catch {
    return null;
  }
}
```

#### Do

```tsx
function Comments({ load }: { load: Promise<string[]> }) {
  const comments = use(load);
  return <ul>{comments.map((c) => <li key={c}>{c}</li>)}</ul>;
}
```

## Differences from the source

- A `return` gives JSX when it returns an element, a parenthesised one, a ternary with an element in a branch, or `x && <El />`; a value that becomes JSX through a variable is not followed.
