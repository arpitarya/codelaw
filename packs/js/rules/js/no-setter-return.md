---
id: js/no-setter-return
title: "A setter does not return a value"
languages: [javascript, jsx]
fix: manual
fix_reason: "whether the returned expression was meant as a side effect needs a reader"
stability: preview
ported_from: [eslint/no-setter-return]
since: 0.1.0
---

A setter body contains no `return <value>`; a bare `return;` to leave early is fine.

**Why:** The value a setter returns is discarded, so returning one hides a mistake.

```codelaw
native: js/no-setter-return
```

## How to fix

1. Keep the expression's side effect as a statement.
2. Remove the value from the `return`, or the `return`.

## Examples

### JavaScript

#### Don't

```js
const box = {
  set size(n) {
    return (this.n = n);
  },
};
```

#### Do

```js
const box = {
  set size(n) {
    this.n = n;
  },
};
```

### JSX

#### Don't

```jsx
const box = {
  set size(n) {
    return (this.n = n);
  },
};
```

#### Do

```jsx
const box = {
  set size(n) {
    this.n = n;
  },
};
```
