---
id: ts/no-empty-object-type
title: "No empty interfaces or {} types"
languages: [typescript, tsx]
fix: manual
fix_reason: "the right replacement — `object`, `unknown` or `Record<string, never>` — depends on intent"
stability: preview
ported_from: [typescript-eslint/no-empty-object-type]
since: 0.1.0
---

Code doesn't declare an empty interface or use the type `{}`.

**Why:** `{}` accepts every value except `null` and `undefined`, which is almost never what an
empty-looking type was meant to say.

```codelaw
match:
  any:
    - kind: interface_declaration
      has: { field: body, regex: '^\{\s*\}$' }
    - kind: object_type
      regex: '^\{\s*\}$'
message: "`{}` means any non-null value, not an empty object; say what you mean"
```

## How to fix

1. For any object, use `object`.
2. For any value, use `unknown`.
3. For an object with no keys, use `Record<string, never>`.

## Examples

### TypeScript

#### Don't

```ts
interface Options {}
type Empty = {};
```

#### Do

```ts
type Options = Record<string, never>;
type Anything = unknown;
```

### TSX

#### Don't

```tsx
interface Options {}
type Empty = {};
```

#### Do

```tsx
type Options = Record<string, never>;
type Anything = unknown;
```
