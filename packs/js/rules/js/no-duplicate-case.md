---
id: js/no-duplicate-case
title: "No two case labels in a switch are the same"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which label was meant is a decision"
stability: preview
ported_from: [eslint/no-duplicate-case]
since: 0.1.0
---

Each `case` in a `switch` tests a different expression.

**Why:** A repeated case can never match: the first one always wins.

```codelaw
native: js/no-duplicate-case
```

## How to fix

1. Find the label that was meant to test something else.
2. Correct it, or merge the two bodies.

## Examples

### JavaScript

#### Don't

```js
switch (status) {
  case 200:
    ok();
    break;
  case 200:
    retry();
    break;
}
```

#### Do

```js
switch (status) {
  case 200:
    ok();
    break;
  case 503:
    retry();
    break;
}
```

### JSX

#### Don't

```jsx
switch (status) {
  case 200:
    ok();
    break;
  case 200:
    retry();
    break;
}
```

#### Do

```jsx
switch (status) {
  case 200:
    ok();
    break;
  case 503:
    retry();
    break;
}
```

### TypeScript

#### Don't

```ts
switch (status) {
  case 200:
    ok();
    break;
  case 200:
    retry();
    break;
}
```

#### Do

```ts
switch (status) {
  case 200:
    ok();
    break;
  case 503:
    retry();
    break;
}
```

### TSX

#### Don't

```tsx
switch (status) {
  case 200:
    ok();
    break;
  case 200:
    retry();
    break;
}
```

#### Do

```tsx
switch (status) {
  case 200:
    ok();
    break;
  case 503:
    retry();
    break;
}
```
