---
id: js/prefer-rest-params
title: "Rest parameters instead of arguments"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the parameter list has to be rewritten by hand"
stability: preview
ported_from: [eslint/prefer-rest-params]
since: 0.1.0
---

Functions take a rest parameter `...args` instead of reading `arguments`.

**Why:** `arguments` is not a real array, isn't available in arrow functions, and hides how many
arguments a function takes.

```codelaw
match:
  kind: identifier
  regex: ^arguments$
where:
  $$MATCH: builtin
message: "Use a rest parameter (`...args`) instead of `arguments`"
```

## How to fix

1. Add a rest parameter `...args` to the function.
2. Replace each use of `arguments` with it.

## Examples

### JavaScript

#### Don't

```js
function sum() {
  return Array.from(arguments).reduce(add, 0);
}
```

#### Do

```js
function sum(...numbers) {
  return numbers.reduce(add, 0);
}
```

### JSX

#### Don't

```jsx
function sum() {
  return Array.from(arguments).reduce(add, 0);
}
```

#### Do

```jsx
function sum(...numbers) {
  return numbers.reduce(add, 0);
}
```

### TypeScript

#### Don't

```ts
function sum() {
  return Array.from(arguments).reduce(add, 0);
}
```

#### Do

```ts
function sum(...numbers) {
  return numbers.reduce(add, 0);
}
```

### TSX

#### Don't

```tsx
function sum() {
  return Array.from(arguments).reduce(add, 0);
}
```

#### Do

```tsx
function sum(...numbers) {
  return numbers.reduce(add, 0);
}
```
