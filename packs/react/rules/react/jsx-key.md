---
id: react/jsx-key
title: "Elements in an array or a map() callback have a key"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "the key must be a stable identity of the item, which only the author knows"
stability: preview
ported_from: [eslint-plugin-react/jsx-key]
since: 0.1.0
---

An element written in an array literal, or returned by a `.map()` callback, has a `key` prop.

**Why:** Without keys, React can't match list items between renders, which loses state and re-
renders more than it must.

```codelaw
match:
  any:
    - kind: jsx_element
    - kind: jsx_self_closing_element
  not:
    any:
      - has: { stopBy: neighbor, kind: jsx_attribute, regex: '^key\b' }
      - has: { field: open_tag, has: { stopBy: neighbor, kind: jsx_attribute, regex: '^key\b' } }
      - has: { stopBy: neighbor, kind: jsx_expression, regex: '^\{\s*\.\.\.' }
      - has: { field: open_tag, has: { stopBy: neighbor, kind: jsx_expression, regex: '^\{\s*\.\.\.' } }
  inside:
    stopBy: neighbor
    any:
      - kind: array
      - kind: arrow_function
        inside:
          stopBy: neighbor
          kind: arguments
          inside: { stopBy: neighbor, kind: call_expression, has: { field: function, kind: member_expression, regex: '\.map$' } }
      - kind: parenthesized_expression
        inside:
          stopBy: neighbor
          kind: arrow_function
          inside:
            stopBy: neighbor
            kind: arguments
            inside: { stopBy: neighbor, kind: call_expression, has: { field: function, kind: member_expression, regex: '\.map$' } }
message: "An element in a list needs a `key` so React can tell its items apart"
```

## How to fix

1. Give the element a `key` that identifies the item, such as its id.
2. Avoid the array index if items can be reordered or removed.

## Examples

### JavaScript

#### Don't

```js
const items = todos.map((todo) => <li>{todo.text}</li>);
```

#### Do

```js
const items = todos.map((todo) => <li key={todo.id}>{todo.text}</li>);
```

### JSX

#### Don't

```jsx
const items = todos.map((todo) => <li>{todo.text}</li>);
```

#### Do

```jsx
const items = todos.map((todo) => <li key={todo.id}>{todo.text}</li>);
```

### TSX

#### Don't

```tsx
const items = todos.map((todo) => <li>{todo.text}</li>);
```

#### Do

```tsx
const items = todos.map((todo) => <li key={todo.id}>{todo.text}</li>);
```

## Differences from the source

Elements returned from a `.map()` callback's block body (`return <li/>`) are not checked yet.
