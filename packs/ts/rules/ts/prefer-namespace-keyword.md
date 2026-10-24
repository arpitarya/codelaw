---
id: ts/prefer-namespace-keyword
title: "namespace, not module, for a TypeScript namespace"
languages: [typescript, tsx]
fix: auto
stability: preview
ported_from: [typescript-eslint/prefer-namespace-keyword]
since: 0.1.0
---

A TypeScript namespace is declared with `namespace`, not `module`; `declare module "x"` is
unaffected.

**Why:** `module Foo {}` is the old spelling and is easy to confuse with an external module
declaration.

```codelaw
match:
  kind: module
  all:
    - has: { field: name, kind: identifier, pattern: $NAME }
    - has: { field: body, pattern: $BODY }
message: "Write `namespace` for a TypeScript namespace; `module` is for declaring external modules"
fix: "namespace $NAME $BODY"
```

## Examples

### TypeScript

#### Don't

```ts
module Shapes {
  export const sides = 4;
}
```

#### Do

```ts
namespace Shapes {
  export const sides = 4;
}
```

### TSX

#### Don't

```tsx
module Shapes {
  export const sides = 4;
}
```

#### Do

```tsx
namespace Shapes {
  export const sides = 4;
}
```
