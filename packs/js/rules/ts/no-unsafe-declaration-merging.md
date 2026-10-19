---
id: ts/no-unsafe-declaration-merging
title: "No interface that merges with a class of the same name"
languages: [typescript, tsx]
fix: manual
fix_reason: "whether to rename or to move the members into the class needs the author"
stability: preview
ported_from: [typescript-eslint/no-unsafe-declaration-merging]
since: 0.1.0
---

An interface doesn't share its name with a class in the same scope.

**Why:** The merged interface adds members to the class's type that the class never initialises, and
the compiler can't tell.

```codelaw
native: ts/no-unsafe-declaration-merging
```

## How to fix

1. Move the interface's members into the class, or rename the interface.

## Examples

### TypeScript

#### Don't

```ts
interface Point {
  z: number;
}
class Point {
  x = 0;
}
```

#### Do

```ts
interface PointLike {
  z: number;
}
class Point {
  x = 0;
}
```

### TSX

#### Don't

```tsx
interface Point {
  z: number;
}
class Point {
  x = 0;
}
```

#### Do

```tsx
interface PointLike {
  z: number;
}
class Point {
  x = 0;
}
```
