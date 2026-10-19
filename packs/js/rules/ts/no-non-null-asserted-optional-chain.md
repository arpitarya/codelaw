---
id: ts/no-non-null-asserted-optional-chain
title: "No non-null assertion after an optional chain"
languages: [typescript, tsx]
fix: suggest
fix_reason: "removing `!` widens the type to include `undefined`, which later code may not handle"
stability: preview
ported_from: [typescript-eslint/no-non-null-asserted-optional-chain]
since: 0.1.0
---

An optional chain (`a?.b`) is not followed by a non-null assertion (`!`).

**Why:** The two contradict each other: `?.` allows `undefined`, and `!` asserts it never happens.

```codelaw
match:
  pattern: $X!
  has:
    stopBy: neighbor
    any: [{ kind: member_expression }, { kind: call_expression }, { kind: subscript_expression }]
    has: { kind: optional_chain, stopBy: neighbor }
message: "`?.` says this can be undefined; `!` says it can't"
fix: "$X"
```

## Examples

### TypeScript

#### Don't

```ts
const name = user?.name!;
```

#### Do

```ts
const name = user?.name;
```

### TSX

#### Don't

```tsx
const name = user?.name!;
```

#### Do

```tsx
const name = user?.name;
```
