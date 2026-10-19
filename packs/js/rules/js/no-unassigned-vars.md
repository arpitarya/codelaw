---
id: js/no-unassigned-vars
title: "A variable that is read is also assigned"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the value the variable should hold is the author's"
stability: preview
ported_from: [eslint/no-unassigned-vars]
since: 0.1.0
---

A variable declared without a value is assigned somewhere before it's read.

**Why:** A variable that is read but never assigned always holds `undefined`, which is a bug or dead
code.

```codelaw
match:
  kind: variable_declarator
  has: { field: name, kind: identifier, pattern: $N }
  inside:
    stopBy: neighbor
    any: [{ kind: lexical_declaration }, { kind: variable_declaration }]
  not:
    any:
      - has: { field: value, pattern: $VALUE }
      - inside: { kind: ambient_declaration, stopBy: end }
where:
  $N: [{ not: reassigned }, { not: unused }]
message: "`$N` is read but never assigned, so it's always undefined"
```

## How to fix

1. Assign the value the variable should hold.
2. If it's meant to be `undefined`, use `undefined` directly.

## Examples

### JavaScript

#### Don't

```js
let status;
report(status);
```

#### Do

```js
let status = "ok";
report(status);
```

### JSX

#### Don't

```jsx
let status;
report(status);
```

#### Do

```jsx
let status = "ok";
report(status);
```

### TypeScript

#### Don't

```ts
let status;
report(status);
```

#### Do

```ts
let status = "ok";
report(status);
```

### TSX

#### Don't

```tsx
let status;
report(status);
```

#### Do

```tsx
let status = "ok";
report(status);
```
