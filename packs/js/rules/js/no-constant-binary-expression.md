---
id: js/no-constant-binary-expression
title: "No binary expression whose result is fixed"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the intended grouping or comparison needs the author"
stability: preview
ported_from: [eslint/no-constant-binary-expression]
since: 0.1.0
---

The left side of `??`, `||` and `&&` is not a value whose nullishness or truthiness is fixed, and
nothing is compared with a newly created object or with `null` when it can never be `null`.

**Why:** Such an expression always gives the same answer, which usually means the author misread
operator precedence.

```codelaw
native: js/no-constant-binary-expression
```

## How to fix

1. Add the parentheses the expression was meant to have.
2. Or compare with the value that was meant.

## Examples

### JavaScript

#### Don't

```js
const total = price + tax ?? 0;
```

#### Do

```js
const total = price + (tax ?? 0);
```

### JSX

#### Don't

```jsx
const total = price + tax ?? 0;
```

#### Do

```jsx
const total = price + (tax ?? 0);
```

### TypeScript

#### Don't

```ts
const total = price + tax ?? 0;
```

#### Do

```ts
const total = price + (tax ?? 0);
```

### TSX

#### Don't

```tsx
const total = price + tax ?? 0;
```

#### Do

```tsx
const total = price + (tax ?? 0);
```
