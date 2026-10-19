---
id: js/no-useless-assignment
title: "No value is stored that nothing reads"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether the store is dead or the read is missing needs the author"
stability: preview
ported_from: [eslint/no-useless-assignment]
since: 0.1.0
---

A variable is not assigned a value that is overwritten before anything reads it, or that nothing
reads after the assignment.

**Why:** A dead store is wasted work and usually means the code reads the wrong variable.

```codelaw
native: js/no-useless-assignment
```

## How to fix

1. If the value is not needed, remove the assignment.
2. If it should be read, read it where it was meant to be used.

## Examples

### JavaScript

#### Don't

```js
function total(items) {
  let sum = 0;
  sum = items.length;
  return sum;
}
```

#### Do

```js
function total(items) {
  const sum = items.length;
  return sum;
}
```

### JSX

#### Don't

```jsx
function total(items) {
  let sum = 0;
  sum = items.length;
  return sum;
}
```

#### Do

```jsx
function total(items) {
  const sum = items.length;
  return sum;
}
```

### TypeScript

#### Don't

```ts
function total(items) {
  let sum = 0;
  sum = items.length;
  return sum;
}
```

#### Do

```ts
function total(items) {
  const sum = items.length;
  return sum;
}
```

### TSX

#### Don't

```tsx
function total(items) {
  let sum = 0;
  sum = items.length;
  return sum;
}
```

#### Do

```tsx
function total(items) {
  const sum = items.length;
  return sum;
}
```

## Differences from the source

Only stores a straight-line reading proves dead are reported: an overwrite in the same block, or a
last assignment outside any loop. A variable a nested function touches is skipped.
