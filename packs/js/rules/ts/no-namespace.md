---
id: ts/no-namespace
title: "No TypeScript namespaces"
languages: [typescript, tsx]
fix: manual
fix_reason: "moving a namespace's members into a module changes every import"
stability: preview
ported_from: [typescript-eslint/no-namespace]
since: 0.1.0
---

Code is organised with ES modules, not `namespace` or `module` blocks; `declare module "x"` and
`declare global` are allowed.

**Why:** Namespaces predate ES modules and don't work with modern bundlers and tooling as well.

```codelaw
match:
  any:
    - kind: internal_module
    - kind: module
      has: { field: name, kind: identifier }
  not:
    inside: { kind: ambient_declaration, stopBy: end }
message: "Use ES modules instead of a namespace"
```

## How to fix

1. Move the namespace's members to a module of their own.
2. Export them, and import them where the namespace was used.

## Examples

### TypeScript

#### Don't

```ts
namespace Geometry {
  export const pi = 3.14;
}
```

#### Do

```ts
export const pi = 3.14;
```

### TSX

#### Don't

```tsx
namespace Geometry {
  export const pi = 3.14;
}
```

#### Do

```tsx
export const pi = 3.14;
```
