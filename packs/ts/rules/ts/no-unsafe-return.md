---
id: ts/no-unsafe-return
title: "No `return` of a value typed `any`"
languages: [typescript, tsx]
fix: manual
fix_reason: "the type the value should have comes from the author"
stability: preview
ported_from: [typescript-eslint/no-unsafe-return]
tags: [types]
since: 0.3.0
---

A function returns only values TypeScript knows the type of.

**Why:** A returned `any` takes the place of the function's declared result: every caller gets an
unchecked value, however carefully the function is typed.

```codelaw
match:
  kind: return_statement
  has: { pattern: $E, stopBy: neighbor }
type:
  $E: any
message: "`$E` is typed `any`; returning it hands every caller an unchecked value"
```

## How to fix

1. Give the value a type before returning it: annotate it, narrow it, or validate it.
2. If the function really returns anything, declare its result `unknown` so callers must check it.

## Examples

### TypeScript

#### Don't

```ts
function load(text: string): number {
  return JSON.parse(text);
}
```

#### Do

```ts
function load(text: string): number {
  const value: unknown = JSON.parse(text);
  return typeof value === "number" ? value : 0;
}
```

### TSX

#### Don't

```tsx
function load(text: string): number {
  return JSON.parse(text);
}
```

#### Do

```tsx
function load(text: string): number {
  const value: unknown = JSON.parse(text);
  return typeof value === "number" ? value : 0;
}
```

## Differences from the source

An arrow function's expression body (`() => value`) is not checked yet, and a function whose result
is declared `any` or `unknown` is reported too; typescript-eslint allows it.
