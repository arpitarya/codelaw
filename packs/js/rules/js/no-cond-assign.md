---
id: js/no-cond-assign
title: "No assignment in a condition, unless doubly parenthesised"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether a comparison or an assignment was meant needs the author"
stability: preview
ported_from: [eslint/no-cond-assign]
since: 0.1.0
---

The condition of an `if`, `while`, `do … while` or `for` is not an assignment, unless it is wrapped
in a second pair of parentheses to say it's meant.

**Why:** `if (a = b)` assigns and tests the result, which is almost always a mistyped `===`.

```codelaw
match:
  kind: assignment_expression
  any:
    - inside:
        stopBy: neighbor
        kind: parenthesized_expression
        inside:
          stopBy: neighbor
          any: [{ kind: if_statement }, { kind: while_statement }, { kind: do_statement }]
    - inside:
        stopBy: neighbor
        kind: expression_statement
        inside: { stopBy: neighbor, kind: for_statement, field: condition }
message: "An assignment in a condition is usually a mistyped comparison"
```

## How to fix

1. If a comparison was meant, use `===`.
2. If the assignment is meant, wrap it in a second pair of parentheses.

## Examples

### JavaScript

#### Don't

```js
if (user = admin) {
  grant();
}
```

#### Do

```js
if (user === admin) {
  grant();
}
```

```js
while ((node = node.next)) {
  visit(node);
}
```

### JSX

#### Don't

```jsx
if (user = admin) {
  grant();
}
```

#### Do

```jsx
if (user === admin) {
  grant();
}
```

```jsx
while ((node = node.next)) {
  visit(node);
}
```

### TypeScript

#### Don't

```ts
if (user = admin) {
  grant();
}
```

#### Do

```ts
if (user === admin) {
  grant();
}
```

```ts
while ((node = node.next)) {
  visit(node);
}
```

### TSX

#### Don't

```tsx
if (user = admin) {
  grant();
}
```

#### Do

```tsx
if (user === admin) {
  grant();
}
```

```tsx
while ((node = node.next)) {
  visit(node);
}
```
