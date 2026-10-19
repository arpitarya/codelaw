---
id: ts/no-extra-non-null-assertion
title: "No repeated non-null assertions"
languages: [typescript, tsx]
fix: auto
stability: preview
ported_from: [typescript-eslint/no-extra-non-null-assertion]
since: 0.1.0
---

A non-null assertion `!` is not repeated.

**Why:** `x!!` means the same as `x!`; the second `!` is a typo.

```codelaw
match:
  pattern: $X!!
message: "One `!` is enough"
fix: "$X!"
```

## Examples

### TypeScript

#### Don't

```ts
const n = value!!;
```

#### Do

```ts
const n = value!;
```

### TSX

#### Don't

```tsx
const n = value!!;
```

#### Do

```tsx
const n = value!;
```
