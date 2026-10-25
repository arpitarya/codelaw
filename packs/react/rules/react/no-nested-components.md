---
id: react/no-nested-components
title: "Components are not defined inside other components"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "moving the component out means passing in the values it closed over as props"
stability: preview
ported_from: [eslint-plugin-react-hooks/static-components, "@eslint-react/no-nested-component-definitions"]
since: 0.3.0
---

A PascalCase function that renders JSX is not defined inside another component — a function
component, or a class component's `render()`. Lower-case helpers that return JSX
(`const renderRow = () => <tr />`) are not components and are fine.

**Why:** A component defined during render is a new component type on every render: React unmounts the old one and mounts the new one, so its state and its DOM are thrown away each time.

```codelaw
native: react/no-nested-components
```

## How to fix

1. Move the component to the top level of the module.
2. Pass the values it read from the outer component as props.

## Examples

### JavaScript

#### Don't

```js
function List({ items }) {
  const Item = ({ text }) => <li>{text}</li>;
  return <ul>{items.map((t) => <Item key={t} text={t} />)}</ul>;
}
```

#### Do

```js
const Item = ({ text }) => <li>{text}</li>;

function List({ items }) {
  return <ul>{items.map((t) => <Item key={t} text={t} />)}</ul>;
}
```

### JSX

#### Don't

```jsx
function List({ items }) {
  function Item({ text }) {
    return <li>{text}</li>;
  }
  return <ul>{items.map((t) => <Item key={t} text={t} />)}</ul>;
}
```

#### Do

```jsx
function Item({ text }) {
  return <li>{text}</li>;
}

function List({ items }) {
  return <ul>{items.map((t) => <Item key={t} text={t} />)}</ul>;
}
```

### TSX

#### Don't

```tsx
function List({ items }: { items: string[] }) {
  const Item = ({ text }: { text: string }) => <li>{text}</li>;
  return <ul>{items.map((t) => <Item key={t} text={t} />)}</ul>;
}
```

#### Do

```tsx
const Item = ({ text }: { text: string }) => <li>{text}</li>;

function List({ items }: { items: string[] }) {
  return <ul>{items.map((t) => <Item key={t} text={t} />)}</ul>;
}
```

## Differences from the source

- The React Compiler's `static-components` reports a component made during render only where it is used as JSX; @eslint-react's `no-nested-component-definitions` reports the definition. codelaw reports the definition, like @eslint-react.
- A component made by a call (`const C = styled(…)`, `const C = memo(…)` with no function written there) or passed inline as a JSX prop is not reported.
