---
id: js/no-ex-assign
title: "A caught error is not reassigned"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "where the original error should go is a decision"
stability: preview
ported_from: [eslint/no-ex-assign]
since: 0.1.0
---

Code doesn't assign a new value to the parameter of a `catch` clause.

**Why:** Reassigning the caught error discards it, so nothing can report or rethrow it.

```codelaw
match:
  kind: identifier
where:
  $$MATCH: [write, { binding: catch-param }]
message: "Assigning to the caught error `$$MATCH` loses it"
```

## How to fix

1. Use a new variable for the new value.
2. Keep the original error, e.g. as `{ cause: err }`.

## Examples

### JavaScript

#### Don't

```js
try {
  load();
} catch (err) {
  err = new Error("failed");
  throw err;
}
```

#### Do

```js
try {
  load();
} catch (err) {
  throw new Error("failed", { cause: err });
}
```

### JSX

#### Don't

```jsx
try {
  load();
} catch (err) {
  err = new Error("failed");
  throw err;
}
```

#### Do

```jsx
try {
  load();
} catch (err) {
  throw new Error("failed", { cause: err });
}
```

### TypeScript

#### Don't

```ts
try {
  load();
} catch (err) {
  err = new Error("failed");
  throw err;
}
```

#### Do

```ts
try {
  load();
} catch (err) {
  throw new Error("failed", { cause: err });
}
```

### TSX

#### Don't

```tsx
try {
  load();
} catch (err) {
  err = new Error("failed");
  throw err;
}
```

#### Do

```tsx
try {
  load();
} catch (err) {
  throw new Error("failed", { cause: err });
}
```
