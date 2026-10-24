---
id: ts/ban-ts-comment
title: "No @ts-ignore or @ts-nocheck; @ts-expect-error says why"
languages: [typescript, tsx]
fix: manual
fix_reason: "the reason the error is expected has to come from the author"
stability: preview
ported_from: [typescript-eslint/ban-ts-comment]
since: 0.1.0
---

Type-checking is never switched off with `@ts-ignore` or `@ts-nocheck`, and a `@ts-expect-error`
says why the error is expected.

**Why:** `@ts-ignore` hides an error for good, even after the code it excused changes; `@ts-expect-
error` fails as soon as the error is gone.

```codelaw
match:
  kind: comment
  any:
    - regex: ^(//|/\*)\s*@ts-(ignore|nocheck)\b
    - regex: ^(//|/\*)\s*@ts-expect-error\s*(\*/)?$
message: "Don't silence the compiler with `@ts-ignore` or `@ts-nocheck`; use `@ts-expect-error: <why>`"
```

## How to fix

1. Fix the type error if you can.
2. Otherwise replace the comment with `// @ts-expect-error: <why>`.

## Examples

### TypeScript

#### Don't

```ts
// @ts-ignore
const n: number = "1";
```

#### Do

```ts
// @ts-expect-error: the fixture checks the compiler's message
const n: number = "1";
```

### TSX

#### Don't

```tsx
// @ts-ignore
const n: number = "1";
```

#### Do

```tsx
// @ts-expect-error: the fixture checks the compiler's message
const n: number = "1";
```
