---
id: js/no-compare-neg-zero
title: "No comparisons against -0"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether `0` or `-0` was meant needs the author"
stability: preview
ported_from: [eslint/no-compare-neg-zero]
since: 0.1.0
---

Code doesn't compare a value with `-0` using `==`, `===`, `<` or the other comparison operators.

**Why:** These operators treat `-0` and `0` as equal, so the comparison doesn't test what it says.

```codelaw
match:
  kind: binary_expression
  any:
    - has: { field: right, pattern: "-0" }
    - has: { field: left, pattern: "-0" }
  has:
    field: operator
    regex: ^(==|===|!=|!==|<|>|<=|>=)$
message: "Comparing with -0 is the same as comparing with 0; use Object.is(x, -0) to test for -0"
```

## How to fix

1. To test for negative zero, use `Object.is(x, -0)`.
2. To test for zero of either sign, compare with `0`.

## Examples

### JavaScript

#### Don't

```js
if (x === -0) {
  flip();
}
```

#### Do

```js
if (Object.is(x, -0)) {
  flip();
}
```

### JSX

#### Don't

```jsx
if (x === -0) {
  flip();
}
```

#### Do

```jsx
if (Object.is(x, -0)) {
  flip();
}
```

### TypeScript

#### Don't

```ts
if (x === -0) {
  flip();
}
```

#### Do

```ts
if (Object.is(x, -0)) {
  flip();
}
```

### TSX

#### Don't

```tsx
if (x === -0) {
  flip();
}
```

#### Do

```tsx
if (Object.is(x, -0)) {
  flip();
}
```
