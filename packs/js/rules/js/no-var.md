---
id: js/no-var
title: "let or const instead of var"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "`let` is block-scoped, so a `var` used outside its block or before its declaration changes meaning"
stability: preview
ported_from: [eslint/no-var]
since: 0.1.0
---

Variables are declared with `let` or `const`, never `var`.

**Why:** `var` is function-scoped and hoisted, which causes bugs `let` and `const` rule out.

```codelaw
native: js/no-var
```

## Examples

### JavaScript

#### Don't

```js
var total = 0;
```

#### Do

```js
let total = 0;
```

### JSX

#### Don't

```jsx
var total = 0;
```

#### Do

```jsx
let total = 0;
```

### TypeScript

#### Don't

```ts
var total = 0;
```

#### Do

```ts
let total = 0;
```

### TSX

#### Don't

```tsx
var total = 0;
```

#### Do

```tsx
let total = 0;
```
