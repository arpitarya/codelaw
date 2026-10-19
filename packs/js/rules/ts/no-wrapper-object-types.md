---
id: ts/no-wrapper-object-types
title: "Primitive types, not wrapper object types"
languages: [typescript, tsx]
fix: auto
stability: preview
ported_from: [typescript-eslint/no-wrapper-object-types]
since: 0.1.0
---

Types use `string`, `number`, `boolean`, `symbol`, `bigint` and `object`, not the wrapper objects
`String`, `Number`, `Boolean`, `Symbol`, `BigInt` and `Object`.

**Why:** The wrapper types describe boxed objects almost no code uses, and accept values the
primitive types reject.

```codelaw
native: ts/no-wrapper-object-types
```

## Examples

### TypeScript

#### Don't

```ts
let label: String = "a";
```

#### Do

```ts
let label: string = "a";
```

### TSX

#### Don't

```tsx
let label: String = "a";
```

#### Do

```tsx
let label: string = "a";
```
