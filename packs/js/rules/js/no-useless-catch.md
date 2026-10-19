---
id: js/no-useless-catch
title: "No catch that only rethrows"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "removing the try block changes the scope of declarations inside it"
stability: preview
ported_from: [eslint/no-useless-catch]
since: 0.1.0
---

A `catch` clause does more than rethrow the error it caught.

**Why:** `catch (e) { throw e; }` does nothing but add noise and a level of nesting.

```codelaw
match:
  kind: catch_clause
  all:
    - has: { field: parameter, pattern: $E }
    - has:
        field: body
        regex: ^\{\s*throw\s+[A-Za-z_$][A-Za-z0-9_$]*\s*;?\s*\}$
        has: { kind: throw_statement, has: { pattern: $E }, stopBy: end }
message: "This catch only rethrows `$E`; remove it"
```

## How to fix

1. If there is no `finally`, replace the `try` statement with the statements of its block.
2. If there is a `finally`, remove only the `catch` clause.

## Examples

### JavaScript

#### Don't

```js
try {
  load();
} catch (e) {
  throw e;
}
```

#### Do

```js
load();
```

### JSX

#### Don't

```jsx
try {
  load();
} catch (e) {
  throw e;
}
```

#### Do

```jsx
load();
```

### TypeScript

#### Don't

```ts
try {
  load();
} catch (e) {
  throw e;
}
```

#### Do

```ts
load();
```

### TSX

#### Don't

```tsx
try {
  load();
} catch (e) {
  throw e;
}
```

#### Do

```tsx
load();
```
