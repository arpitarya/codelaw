---
id: ts/no-unused-expressions
title: "No expression statements that do nothing"
languages: [typescript, tsx]
fix: manual
fix_reason: "whether a call, an assignment or nothing was meant needs the author"
stability: preview
ported_from: [typescript-eslint/no-unused-expressions, eslint/no-unused-expressions]
since: 0.1.0
---

Every expression statement does something: a call, an assignment, an update, `await`, `yield`,
`delete` or `void`. Directives such as `"use strict"` are allowed.

**Why:** An expression whose value is thrown away is usually a forgotten call or assignment.

```codelaw
native: ts/no-unused-expressions
```

## How to fix

1. Add the call or assignment that was meant, or remove the statement.

## Examples

### TypeScript

#### Don't

```ts
function close(conn: Connection) {
  conn.close;
}
```

#### Do

```ts
function close(conn: Connection) {
  conn.close();
}
```

### TSX

#### Don't

```tsx
function close(conn: Connection) {
  conn.close;
}
```

#### Do

```tsx
function close(conn: Connection) {
  conn.close();
}
```
