---
id: js/use-isnan
title: "Test for NaN with Number.isNaN"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the comparison's operator decides whether to negate"
stability: preview
ported_from: [eslint/use-isnan]
since: 0.1.0
---

Code tests for `NaN` with `Number.isNaN(x)`, never by comparing with `NaN`.

**Why:** `NaN` is not equal to anything, itself included, so `x === NaN` is always false.

```codelaw
match:
  kind: binary_expression
  has:
    field: operator
    regex: ^(==|===|!=|!==|<|>|<=|>=)$
  any:
    - has: { field: left, regex: '^(Number\.)?NaN$' }
    - has: { field: right, regex: '^(Number\.)?NaN$' }
message: "Nothing equals NaN, not even NaN; use Number.isNaN(x)"
```

## How to fix

1. Replace `x === NaN` or `x == NaN` with `Number.isNaN(x)`.
2. Replace `x !== NaN` or `x != NaN` with `!Number.isNaN(x)`.

## Examples

### JavaScript

#### Don't

```js
if (price === NaN) {
  reset();
}
```

#### Do

```js
if (Number.isNaN(price)) {
  reset();
}
```

### JSX

#### Don't

```jsx
if (price === NaN) {
  reset();
}
```

#### Do

```jsx
if (Number.isNaN(price)) {
  reset();
}
```

### TypeScript

#### Don't

```ts
if (price === NaN) {
  reset();
}
```

#### Do

```ts
if (Number.isNaN(price)) {
  reset();
}
```

### TSX

#### Don't

```tsx
if (price === NaN) {
  reset();
}
```

#### Do

```tsx
if (Number.isNaN(price)) {
  reset();
}
```
