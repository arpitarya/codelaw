---
id: react/exhaustive-deps
title: "Effect and memo dependency lists name every value they read"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "adding a dependency can make an effect run more often"
stability: preview
ported_from: [eslint-plugin-react-hooks/exhaustive-deps]
since: 0.1.0
---

The dependency list of `useEffect`, `useLayoutEffect`, `useCallback`, `useMemo` and
`useImperativeHandle` names every prop, state value and local the callback reads, and nothing from
outside the component.

**Why:** A missing dependency makes the hook keep a stale value; an outside value in the list does
nothing.

```codelaw
native: react/exhaustive-deps
```

## Examples

### JavaScript

#### Don't

```js
function Search({ query }) {
  useEffect(() => {
    fetchResults(query);
  }, []);
}
```

#### Do

```js
function Search({ query }) {
  useEffect(() => {
    fetchResults(query);
  }, [query]);
}
```

### JSX

#### Don't

```jsx
function Search({ query }) {
  useEffect(() => {
    fetchResults(query);
  }, []);
}
```

#### Do

```jsx
function Search({ query }) {
  useEffect(() => {
    fetchResults(query);
  }, [query]);
}
```

### TypeScript

#### Don't

```ts
function Search({ query }) {
  useEffect(() => {
    fetchResults(query);
  }, []);
}
```

#### Do

```ts
function Search({ query }) {
  useEffect(() => {
    fetchResults(query);
  }, [query]);
}
```

### TSX

#### Don't

```tsx
function Search({ query }) {
  useEffect(() => {
    fetchResults(query);
  }, []);
}
```

#### Do

```tsx
function Search({ query }) {
  useEffect(() => {
    fetchResults(query);
  }, [query]);
}
```

## Differences from the source

State setters, reducer dispatchers and refs are treated as stable, as upstream does. Dependencies
are compared by their root name (`props` covers `props.id`).
