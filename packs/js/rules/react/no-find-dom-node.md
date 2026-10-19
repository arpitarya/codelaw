---
id: react/no-find-dom-node
title: "No findDOMNode"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "a ref has to be attached to the right element by hand"
stability: preview
ported_from: [eslint-plugin-react/no-find-dom-node]
since: 0.1.0
---

Code doesn't call `findDOMNode`.

**Why:** `findDOMNode` breaks component abstraction, doesn't work with function components, and is
removed in React 19.

```codelaw
match:
  any:
    - pattern: findDOMNode($$$A)
    - pattern: $R.findDOMNode($$$A)
message: "`findDOMNode` is deprecated; use a ref"
```

## How to fix

1. Attach a ref to the element you need (`ref={this.ref}`).
2. Read `ref.current` instead of calling `findDOMNode`.

## Examples

### JavaScript

#### Don't

```js
const el = ReactDOM.findDOMNode(this);
```

#### Do

```js
const el = this.ref.current;
```

### JSX

#### Don't

```jsx
const el = ReactDOM.findDOMNode(this);
```

#### Do

```jsx
const el = this.ref.current;
```

### TSX

#### Don't

```tsx
const el = ReactDOM.findDOMNode(this);
```

#### Do

```tsx
const el = this.ref.current;
```
