---
id: fixture/ts-no-var
title: L3 fixture rule
languages: [typescript, tsx]
fix: auto
stability: stable
---

A fixture rule for the L3 language matrix (SR-LAW-3); it checks nothing real.

**Why:** every language must be checkable alone and together.

```codelaw
match:
  kind: variable_declaration
  has: { kind: variable_declarator, pattern: $D }
message: "Use let instead of var"
fix: "let $D;"
```

## Examples

### TypeScript

#### Don't

```ts
var a: number = 1;
```

#### Do

```ts
let a: number = 1;
```

### TSX

#### Don't

```tsx
var el = <A />;
```

#### Do

```tsx
let el = <A />;
```
