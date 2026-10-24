---
id: ts/no-misused-new
title: "No constructors declared in interfaces, and no `new` methods in classes"
languages: [typescript, tsx]
fix: manual
fix_reason: "the intended constructor signature needs the author"
stability: preview
ported_from: [typescript-eslint/no-misused-new]
since: 0.1.0
---

An interface doesn't declare a method named `constructor`, and a class doesn't declare a method
named `new`.

**Why:** Both look like constructors but are ordinary methods; constructors in types are written
`new (): T`.

```codelaw
match:
  any:
    - kind: method_signature
      has: { field: name, regex: ^constructor$ }
      inside: { kind: interface_body, stopBy: neighbor }
    - kind: method_definition
      has: { field: name, regex: ^new$ }
message: "This declares a constructor where TypeScript reads a plain method"
```

## How to fix

1. In an interface, describe a constructor with `new (…): T`.
2. In a class, use `constructor(…)`.

## Examples

### TypeScript

#### Don't

```ts
interface Point {
  constructor(): void;
}
```

#### Do

```ts
interface PointConstructor {
  new (): object;
}
```

### TSX

#### Don't

```tsx
interface Point {
  constructor(): void;
}
```

#### Do

```tsx
interface PointConstructor {
  new (): object;
}
```
