---
id: js/no-shadow-restricted-names
title: "No declaring undefined, NaN, Infinity, arguments, eval or globalThis"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the new name is the author's choice"
stability: preview
ported_from: [eslint/no-shadow-restricted-names]
since: 0.1.0
---

No variable, parameter or function is named `undefined`, `NaN`, `Infinity`, `arguments`, `eval` or
`globalThis`.

**Why:** Shadowing these names makes code that relies on them behave in confusing ways.

```codelaw
match:
  kind: identifier
  regex: ^(undefined|NaN|Infinity|arguments|eval|globalThis)$
where:
  $$MATCH: declaration
message: "`$$MATCH` is a restricted name; don't declare it"
```

## How to fix

1. Rename the declaration and its uses.

## Examples

### JavaScript

#### Don't

```js
function check(NaN) {
  return NaN;
}
```

#### Do

```js
function check(value) {
  return value;
}
```

### JSX

#### Don't

```jsx
function check(NaN) {
  return NaN;
}
```

#### Do

```jsx
function check(value) {
  return value;
}
```

### TypeScript

#### Don't

```ts
function check(NaN) {
  return NaN;
}
```

#### Do

```ts
function check(value) {
  return value;
}
```

### TSX

#### Don't

```tsx
function check(NaN) {
  return NaN;
}
```

#### Do

```tsx
function check(value) {
  return value;
}
```
