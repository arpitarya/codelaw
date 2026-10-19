---
id: js/no-self-assign
title: "No assigning a variable to itself"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "assigning a property to itself can run a setter, so removing it may change behaviour"
stability: preview
ported_from: [eslint/no-self-assign]
since: 0.1.0
---

An assignment's left and right sides are not the same expression.

**Why:** `a = a` has no effect; it is usually a typo for another name.

```codelaw
match:
  pattern: $A = $A
message: "Assigning `$A` to itself does nothing"
```

## How to fix

1. If another value was meant, assign that.
2. Otherwise remove the assignment.

## Examples

### JavaScript

#### Don't

```js
let x = 1;
x = x;
```

#### Do

```js
let x = 1;
x = y;
```

### JSX

#### Don't

```jsx
let x = 1;
x = x;
```

#### Do

```jsx
let x = 1;
x = y;
```

### TypeScript

#### Don't

```ts
let x = 1;
x = x;
```

#### Do

```ts
let x = 1;
x = y;
```

### TSX

#### Don't

```tsx
let x = 1;
x = x;
```

#### Do

```tsx
let x = 1;
x = y;
```
