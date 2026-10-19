---
id: js/no-global-assign
title: "No assignment to built-in globals"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the intended variable needs a name of its own"
stability: preview
ported_from: [eslint/no-global-assign]
since: 0.1.0
---

Code doesn't assign to JavaScript's built-in globals such as `Object`, `undefined` or `NaN`.

**Why:** Built-in globals are read-only or shared by every module, so assigning to one is a bug or a
hazard.

```codelaw
match:
  kind: identifier
where:
  $$MATCH: [write, builtin]
message: "`$$MATCH` is a built-in global; don't assign to it"
```

## How to fix

1. Declare a local variable with its own name.

## Examples

### JavaScript

#### Don't

```js
Object = null;
```

#### Do

```js
const object = null;
```

### JSX

#### Don't

```jsx
Object = null;
```

#### Do

```jsx
const object = null;
```

### TypeScript

#### Don't

```ts
Object = null;
```

#### Do

```ts
const object = null;
```

### TSX

#### Don't

```tsx
Object = null;
```

#### Do

```tsx
const object = null;
```
