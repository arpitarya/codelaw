---
id: ts/no-unsafe-call
title: "No call of a value typed `any`"
languages: [typescript, tsx]
fix: manual
fix_reason: "the type the value should have comes from the author"
stability: preview
ported_from: [typescript-eslint/no-unsafe-call]
tags: [types]
since: 0.3.0
---

A function is called only through a value TypeScript knows the type of.

**Why:** Calling a value typed `any` switches off every check on the call: the arguments, the
result and whether the value is a function at all.

```codelaw
match:
  kind: call_expression
  has: { field: function, pattern: $F }
type:
  $F: any
message: "`$F` is typed `any`, so this call is not type-checked"
```

## How to fix

1. Give the value a type: annotate it, or narrow it with a check before the call.
2. If it comes from an untyped library, add a declaration (`.d.ts`) for it.

## Examples

### TypeScript

#### Don't

```ts
declare const handler: any;
handler(1);
```

#### Do

```ts
declare const handler: (n: number) => void;
handler(1);
```

### TSX

#### Don't

```tsx
declare const handler: any;
handler(1);
```

#### Do

```tsx
declare const handler: (n: number) => void;
handler(1);
```

## Differences from the source

`new` on a value typed `any` and a tagged template on one are not checked yet.
