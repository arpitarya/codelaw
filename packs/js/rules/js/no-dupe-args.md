---
id: js/no-dupe-args
title: "No two parameters share a name"
languages: [javascript, jsx]
fix: manual
fix_reason: "which parameter was meant can't be known from the code"
stability: preview
ported_from: [eslint/no-dupe-args]
since: 0.1.0
---

A function's parameters, destructured ones included, all have different names.

**Why:** The later parameter silently hides the earlier one, so one argument can never be read.

```codelaw
native: js/no-dupe-args
```

## How to fix

1. Rename the duplicate parameter.
2. Update the body to read the parameter it meant.

## Examples

### JavaScript

#### Don't

```js
function add(a, b, a) {
  return a + b;
}
```

#### Do

```js
function add(a, b, c) {
  return a + b + c;
}
```

### JSX

#### Don't

```jsx
function add(a, b, a) {
  return a + b;
}
```

#### Do

```jsx
function add(a, b, c) {
  return a + b + c;
}
```
