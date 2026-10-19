---
id: react/no-render-return-value
title: "The return value of ReactDOM.render is not used"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "code that used the instance needs a ref instead"
stability: preview
ported_from: [eslint-plugin-react/no-render-return-value]
since: 0.1.0
---

The value `ReactDOM.render` returns is not stored, returned or passed on.

**Why:** It's a legacy handle that is `null` in some cases and gone in newer APIs.

```codelaw
match:
  pattern: ReactDOM.render($$$A)
  inside:
    stopBy: neighbor
    any:
      - kind: variable_declarator
      - kind: return_statement
      - kind: assignment_expression
      - kind: arguments
message: "Don't use what `ReactDOM.render` returns; use a ref"
```

## How to fix

1. Pass a ref to the root element and use it instead of the return value.

## Examples

### JavaScript

#### Don't

```js
const app = ReactDOM.render(<App />, root);
```

#### Do

```js
ReactDOM.render(<App ref={appRef} />, root);
```

### JSX

#### Don't

```jsx
const app = ReactDOM.render(<App />, root);
```

#### Do

```jsx
ReactDOM.render(<App ref={appRef} />, root);
```

### TSX

#### Don't

```tsx
const app = ReactDOM.render(<App />, root);
```

#### Do

```tsx
ReactDOM.render(<App ref={appRef} />, root);
```
