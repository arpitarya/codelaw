---
id: js/prefer-const
title: "const for variables that are never reassigned"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: [eslint/prefer-const]
since: 0.1.0
---

A `let` whose names all get their value in the declaration and are never reassigned is a `const`.

**Why:** `const` tells the reader the binding won't change, and the engine enforces it.

```codelaw
native: js/prefer-const
```

## Examples

### JavaScript

#### Don't

```js
let limit = 10;
use(limit);
```

#### Do

```js
const limit = 10;
use(limit);
```

### JSX

#### Don't

```jsx
let limit = 10;
use(limit);
```

#### Do

```jsx
const limit = 10;
use(limit);
```

### TypeScript

#### Don't

```ts
let limit = 10;
use(limit);
```

#### Do

```ts
const limit = 10;
use(limit);
```

### TSX

#### Don't

```tsx
let limit = 10;
use(limit);
```

#### Do

```tsx
const limit = 10;
use(limit);
```
