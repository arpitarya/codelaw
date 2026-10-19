---
id: js/no-debugger
title: "No debugger statements"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: [eslint/no-debugger]
since: 0.1.0
---

Code contains no `debugger` statements.

**Why:** `debugger` pauses execution wherever developer tools are open, which nobody wants in
shipped code.

```codelaw
match:
  kind: debugger_statement
message: "Remove `debugger` before committing"
fix: [remove]
```

## Examples

### JavaScript

#### Don't

```js
function load() {
  debugger;
  return fetchAll();
}
```

#### Do

```js
function load() {
  return fetchAll();
}
```

### JSX

#### Don't

```jsx
function load() {
  debugger;
  return fetchAll();
}
```

#### Do

```jsx
function load() {
  return fetchAll();
}
```

### TypeScript

#### Don't

```ts
function load() {
  debugger;
  return fetchAll();
}
```

#### Do

```ts
function load() {
  return fetchAll();
}
```

### TSX

#### Don't

```tsx
function load() {
  debugger;
  return fetchAll();
}
```

#### Do

```tsx
function load() {
  return fetchAll();
}
```
