---
id: ts/no-unnecessary-type-constraint
title: "No extends any or extends unknown on a type parameter"
languages: [typescript, tsx]
fix: suggest
fix_reason: "in a .tsx arrow function `<T>` alone parses as JSX and needs `<T,>`"
stability: preview
ported_from: [typescript-eslint/no-unnecessary-type-constraint]
since: 0.1.0
---

A type parameter isn't constrained to `any` or `unknown`.

**Why:** Every type extends both, so the constraint says nothing.

```codelaw
match:
  kind: type_parameter
  all:
    - has: { field: name, pattern: $NAME }
    - has:
        field: constraint
        has: { kind: predefined_type, regex: ^(any|unknown)$ }
message: "Every type already extends `any` and `unknown`; drop the constraint"
fix: "$NAME"
```

## Examples

### TypeScript

#### Don't

```ts
function id<T extends unknown>(x: T) {
  return x;
}
```

#### Do

```ts
function id<T>(x: T) {
  return x;
}
```

### TSX

#### Don't

```tsx
function id<T extends unknown>(x: T) {
  return x;
}
```

#### Do

```tsx
function id<T>(x: T) {
  return x;
}
```
