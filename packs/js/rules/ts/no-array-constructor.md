---
id: ts/no-array-constructor
title: "No Array constructor with several arguments"
languages: [typescript, tsx]
fix: auto
stability: preview
ported_from: [typescript-eslint/no-array-constructor, eslint/no-array-constructor]
since: 0.1.0
---

Arrays are built with literals; `new Array(n)` with one argument (a length) is allowed.

**Why:** `new Array(a, b)` and `new Array(n)` mean different things, and the literal says what it
means.

```codelaw
match:
  any:
    - pattern: new Array($$$ARGS)
    - pattern: Array($$$ARGS)
  not:
    any:
      - pattern: new Array($ONE)
      - pattern: Array($ONE)
      - has: { field: type_arguments, pattern: $T }
message: "Use an array literal instead of the Array constructor"
fix: "[$$$ARGS]"
```

## Examples

### TypeScript

#### Don't

```ts
const xs = new Array(1, 2, 3);
```

#### Do

```ts
const xs = [1, 2, 3];
```

### TSX

#### Don't

```tsx
const xs = new Array(1, 2, 3);
```

#### Do

```tsx
const xs = [1, 2, 3];
```
