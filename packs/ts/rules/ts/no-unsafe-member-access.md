---
id: ts/no-unsafe-member-access
title: "No member access on a value typed `any`"
languages: [typescript, tsx]
fix: manual
fix_reason: "the type the value should have comes from the author"
stability: preview
ported_from: [typescript-eslint/no-unsafe-member-access]
tags: [types]
since: 0.3.0
---

A property is read or written only on a value TypeScript knows the type of.

**Why:** Every property of a value typed `any` is `any` too, so one untyped value spreads unchecked
code through everything that reads from it.

```codelaw
match:
  any: [{ kind: member_expression }, { kind: subscript_expression }]
  has:
    field: object
    pattern: $O
    any:
      - not: { any: [{ kind: member_expression }, { kind: subscript_expression }] }
      - all:
          - any: [{ kind: member_expression }, { kind: subscript_expression }]
          - has: { field: object, pattern: $INNER }
type:
  $O: any
  $INNER: { not: any }
message: "`$O` is typed `any`, so `$$MATCH` is not type-checked"
```

## How to fix

1. Give the value a type: annotate it, or narrow it with a check before reading from it.
2. If it comes from `JSON.parse` or another untyped source, validate it into a typed value first.

## Examples

### TypeScript

#### Don't

```ts
declare const config: any;
const port = config.port;
```

#### Do

```ts
declare const config: { port: number };
const port = config.port;
```

### TSX

#### Don't

```tsx
declare const config: any;
const port = config.port;
```

#### Do

```tsx
declare const config: { port: number };
const port = config.port;
```

## Differences from the source

In a chain on an `any` value (`a.b.c`), only the first access is reported, as typescript-eslint does.
