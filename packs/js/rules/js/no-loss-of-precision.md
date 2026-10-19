---
id: js/no-loss-of-precision
title: "Number literals keep their value"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether to use a BigInt, a string or a rounder value depends on how the number is used"
stability: preview
ported_from: [eslint/no-loss-of-precision]
since: 0.1.0
---

A number literal can be represented exactly as a JavaScript number.

**Why:** A literal with more precision than a double holds is silently rounded, so the code doesn't
mean what it says.

```codelaw
native: js/no-loss-of-precision
```

## How to fix

1. Use a BigInt (`123n`) for an exact large integer.
2. Or keep the value as a string, or write the value the code really gets.

## Examples

### JavaScript

#### Don't

```js
const id = 9007199254740993;
```

#### Do

```js
const id = 9007199254740993n;
```

### JSX

#### Don't

```jsx
const id = 9007199254740993;
```

#### Do

```jsx
const id = 9007199254740993n;
```

### TypeScript

#### Don't

```ts
const id = 9007199254740993;
```

#### Do

```ts
const id = 9007199254740993n;
```

### TSX

#### Don't

```tsx
const id = 9007199254740993;
```

#### Do

```tsx
const id = 9007199254740993n;
```
