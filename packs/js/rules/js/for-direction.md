---
id: js/for-direction
title: "A for loop's counter moves toward its end condition"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether the condition or the update is wrong needs the author"
stability: preview
ported_from: [eslint/for-direction]
since: 0.1.0
---

A `for` loop whose condition tests `i < n` counts up, and one that tests `i > n` counts down.

**Why:** A counter that moves the wrong way never meets its end condition, so the loop never ends.

```codelaw
match:
  kind: for_statement
  any:
    - all:
        - has: { field: condition, any: [{ pattern: $I < $END }, { pattern: $I <= $END }] }
        - has: { field: increment, any: [{ pattern: $I-- }, { pattern: --$I }] }
    - all:
        - has: { field: condition, any: [{ pattern: $I > $END }, { pattern: $I >= $END }] }
        - has: { field: increment, any: [{ pattern: $I++ }, { pattern: ++$I }] }
message: "The counter `$I` moves away from the condition that ends this loop"
```

## How to fix

1. Decide which way the loop should count.
2. Fix the update (`i++` or `i--`) or the comparison to match.

## Examples

### JavaScript

#### Don't

```js
for (let i = 0; i < 10; i--) {
  draw(i);
}
```

#### Do

```js
for (let i = 0; i < 10; i++) {
  draw(i);
}
```

### JSX

#### Don't

```jsx
for (let i = 0; i < 10; i--) {
  draw(i);
}
```

#### Do

```jsx
for (let i = 0; i < 10; i++) {
  draw(i);
}
```

### TypeScript

#### Don't

```ts
for (let i = 0; i < 10; i--) {
  draw(i);
}
```

#### Do

```ts
for (let i = 0; i < 10; i++) {
  draw(i);
}
```

### TSX

#### Don't

```tsx
for (let i = 0; i < 10; i--) {
  draw(i);
}
```

#### Do

```tsx
for (let i = 0; i < 10; i++) {
  draw(i);
}
```
