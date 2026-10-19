---
id: react/display-name
title: "A component passed to memo or forwardRef has a name"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "the name is the author's choice"
stability: preview
ported_from: [eslint-plugin-react/display-name]
since: 0.1.0
---

A component wrapped in `memo` or `forwardRef` is a named function, so it has a display name.

**Why:** An anonymous component shows up as `Anonymous` in React DevTools and in error messages,
which makes both harder to read.

```codelaw
match:
  any:
    - pattern: memo($F)
    - pattern: React.memo($F)
    - pattern: forwardRef($F)
    - pattern: React.forwardRef($F)
constraints:
  F:
    any:
      - kind: arrow_function
      - kind: function_expression
        not: { has: { field: name, pattern: $NAME } }
message: "This component has no name, so React DevTools and error messages show it as `Anonymous`"
```

## How to fix

1. Pass a named function: `memo(function Card(props) { … })`.
2. Or set `Card.displayName = "Card"`.

## Examples

### JavaScript

#### Don't

```js
export const Card = memo((props) => <div>{props.title}</div>);
```

#### Do

```js
export const Card = memo(function Card(props) {
  return <div>{props.title}</div>;
});
```

### JSX

#### Don't

```jsx
export const Card = memo((props) => <div>{props.title}</div>);
```

#### Do

```jsx
export const Card = memo(function Card(props) {
  return <div>{props.title}</div>;
});
```

### TSX

#### Don't

```tsx
export const Card = memo((props) => <div>{props.title}</div>);
```

#### Do

```tsx
export const Card = memo(function Card(props) {
  return <div>{props.title}</div>;
});
```

## Differences from the source

Components defined some other way (`createReactClass`, anonymous default exports) are not checked
yet.
