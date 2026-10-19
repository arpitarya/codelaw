---
id: js/no-unreachable
title: "No code after a return, throw, break or continue"
languages: [javascript, jsx]
fix: suggest
fix_reason: "deleting the code is right only when it was really dead, not misplaced"
stability: preview
ported_from: [eslint/no-unreachable]
since: 0.1.0
---

Statements after an unconditional `return`, `throw`, `break` or `continue` in the same block are
removed or moved; hoisted function declarations and uninitialised `var`s are allowed.

**Why:** Unreachable code never runs, so it misleads every reader who assumes it does.

```codelaw
native: js/no-unreachable
```

## Examples

### JavaScript

#### Don't

```js
function total(items) {
  return items.length;
  log(items);
}
```

#### Do

```js
function total(items) {
  return items.length;
}
```

### JSX

#### Don't

```jsx
function total(items) {
  return items.length;
  log(items);
}
```

#### Do

```jsx
function total(items) {
  return items.length;
}
```
