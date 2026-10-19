---
id: react/no-string-refs
title: "No string refs"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "the ref object has to be created where the component can reach it"
stability: preview
ported_from: [eslint-plugin-react/no-string-refs]
since: 0.1.0
---

Refs are objects or callbacks, never strings (`ref="input"`, `this.refs.input`).

**Why:** String refs are legacy, slow, and removed in React 19.

```codelaw
match:
  any:
    - kind: jsx_attribute
      regex: ^ref=["']
    - pattern: this.refs.$X
message: "String refs are legacy; use `useRef`, `createRef` or a callback ref"
```

## How to fix

1. Create a ref with `useRef` or `createRef`.
2. Pass it as `ref={…}` and read `.current`.

## Examples

### JavaScript

#### Don't

```js
const a = <input ref="name" />;
```

#### Do

```js
const a = <input ref={nameRef} />;
```

### JSX

#### Don't

```jsx
const a = <input ref="name" />;
```

#### Do

```jsx
const a = <input ref={nameRef} />;
```

### TSX

#### Don't

```tsx
const a = <input ref="name" />;
```

#### Do

```tsx
const a = <input ref={nameRef} />;
```
