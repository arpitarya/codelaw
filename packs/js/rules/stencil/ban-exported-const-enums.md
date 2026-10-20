---
id: stencil/ban-exported-const-enums
title: "No exported const enums"
languages: [typescript, tsx]
fix: auto
stability: preview
ported_from: [stencil/ban-exported-const-enums]
since: 0.1.0
---

An exported enum is a plain `enum`, not a `const enum`.

**Why:** A `const enum` is erased at compile time, so code that imports it from the built bundle
finds nothing.

```codelaw
native: stencil/ban-exported-const-enums
```

## Examples

### TypeScript

#### Don't

```ts
export const enum Size {
  Small,
  Large,
}
```

#### Do

```ts
export enum Size {
  Small,
  Large,
}
```

### TSX

#### Don't

```tsx
export const enum Size {
  Small,
  Large,
}
```

#### Do

```tsx
export enum Size {
  Small,
  Large,
}
```
