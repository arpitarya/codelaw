---
id: js/no-dupe-else-if
title: "No two conditions in an if/else-if chain are the same"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what the second condition should have been is a decision"
stability: preview
ported_from: [eslint/no-dupe-else-if]
since: 0.1.0
---

Each condition in an `if … else if …` chain is different from the ones before it.

**Why:** A repeated condition's branch can never run, because the earlier one already took it.

```codelaw
native: js/no-dupe-else-if
```

## How to fix

1. Find the repeated condition.
2. Correct it to the case it was meant to handle, or remove the dead branch.

## Examples

### JavaScript

#### Don't

```js
if (isAdmin) {
  grant();
} else if (isAdmin) {
  audit();
}
```

#### Do

```js
if (isAdmin) {
  grant();
} else if (isAuditor) {
  audit();
}
```

### JSX

#### Don't

```jsx
if (isAdmin) {
  grant();
} else if (isAdmin) {
  audit();
}
```

#### Do

```jsx
if (isAdmin) {
  grant();
} else if (isAuditor) {
  audit();
}
```

### TypeScript

#### Don't

```ts
if (isAdmin) {
  grant();
} else if (isAdmin) {
  audit();
}
```

#### Do

```ts
if (isAdmin) {
  grant();
} else if (isAuditor) {
  audit();
}
```

### TSX

#### Don't

```tsx
if (isAdmin) {
  grant();
} else if (isAdmin) {
  audit();
}
```

#### Do

```tsx
if (isAdmin) {
  grant();
} else if (isAuditor) {
  audit();
}
```
