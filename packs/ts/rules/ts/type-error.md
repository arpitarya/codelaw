---
id: ts/type-error
title: "TypeScript's own errors"
languages: [typescript, tsx]
fix: manual
fix_reason: "TypeScript 7.1's API offers no code fixes yet, so each error is fixed by hand"
stability: preview
tags: [types]
since: 0.3.0
---

A file has none of the errors the project's own TypeScript reports for it.

**Why:** These are the errors `tsc` would stop on; reporting them with the rest of the check means
one run, one report, and no second tool to wire up.

```codelaw
native: ts-type-error
message: "TypeScript reports an error here"
```

## How to fix

1. Read the TypeScript error code and message in the finding (`TS2322: …`).
2. Fix the code, or the type it is checked against, the way `tsc` would ask you to.

## Examples

### TypeScript

#### Don't

```ts
export const port: number = "8080";
```

#### Do

```ts
export const port: number = 8080;
```

### TSX

#### Don't

```tsx
export const port: number = "8080";
```

#### Do

```tsx
export const port: number = 8080;
```
