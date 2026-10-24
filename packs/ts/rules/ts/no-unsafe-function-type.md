---
id: ts/no-unsafe-function-type
title: "No Function type"
languages: [typescript, tsx]
fix: manual
fix_reason: "the right signature needs the author"
stability: preview
ported_from: [typescript-eslint/no-unsafe-function-type]
since: 0.1.0
---

The built-in `Function` type is not used as a type.

**Why:** `Function` accepts any function and its calls are unchecked, like `any` for functions.

```codelaw
match:
  kind: type_identifier
  regex: ^Function$
where:
  $$MATCH: builtin
message: "`Function` accepts any function and calls it unchecked; write the signature"
```

## How to fix

1. Write the function's real signature, e.g. `(event: Event) => void`.

## Examples

### TypeScript

#### Don't

```ts
let handler: Function;
```

#### Do

```ts
let handler: (event: Event) => void;
```

### TSX

#### Don't

```tsx
let handler: Function;
```

#### Do

```tsx
let handler: (event: Event) => void;
```
