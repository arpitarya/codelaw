---
id: js/prefer-spread
title: "Spread instead of .apply()"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: [eslint/prefer-spread]
since: 0.1.0
---

A function is called with spread arguments, `f(...args)`, instead of `f.apply(null, args)`.

**Why:** Spread says the same thing more directly, and doesn't need a `this` value.

```codelaw
match:
  any:
    - pattern: $F.apply(null, $ARGS)
    - pattern: $F.apply(undefined, $ARGS)
constraints:
  F: { kind: identifier }
message: "Use spread: `$F(...$ARGS)`"
fix: "$F(...$ARGS)"
```

## Examples

### JavaScript

#### Don't

```js
const max = Math.max.apply(null, values);
const min = least.apply(null, values);
```

#### Do

```js
const max = Math.max.apply(null, values);
const min = least(...values);
```

### JSX

#### Don't

```jsx
const max = Math.max.apply(null, values);
const min = least.apply(null, values);
```

#### Do

```jsx
const max = Math.max.apply(null, values);
const min = least(...values);
```

### TypeScript

#### Don't

```ts
const max = Math.max.apply(null, values);
const min = least.apply(null, values);
```

#### Do

```ts
const max = Math.max.apply(null, values);
const min = least(...values);
```

### TSX

#### Don't

```tsx
const max = Math.max.apply(null, values);
const min = least.apply(null, values);
```

#### Do

```tsx
const max = Math.max.apply(null, values);
const min = least(...values);
```

## Differences from the source

Only `f.apply(null, args)` and `f.apply(undefined, args)` on a plain function name are checked;
method calls are left alone.
