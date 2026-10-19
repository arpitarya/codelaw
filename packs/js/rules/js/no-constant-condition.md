---
id: js/no-constant-condition
title: "No conditions that are always true or always false"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether to remove the branch or fix the condition needs the author"
stability: preview
ported_from: [eslint/no-constant-condition]
since: 0.1.0
---

The condition of an `if`, `while` or `do … while` is not a literal or another value that is always
truthy or always falsy; `while (true)` is allowed.

**Why:** A constant condition is usually leftover debugging or a typo, and hides dead code.

```codelaw
match:
  kind: parenthesized_expression
  inside:
    stopBy: neighbor
    any: [{ kind: if_statement }, { kind: while_statement }, { kind: do_statement }]
    field: condition
  has:
    stopBy: neighbor
    any:
      - kind: number
      - kind: string
      - kind: "true"
      - kind: "false"
      - kind: "null"
      - kind: undefined
      - kind: array
      - kind: object
      - kind: arrow_function
      - kind: function_expression
      - kind: regex
  not:
    all:
      - inside: { stopBy: neighbor, kind: while_statement }
      - has: { stopBy: neighbor, kind: "true" }
message: "This condition is a constant, so the branch always or never runs"
```

## How to fix

1. If the branch is dead, remove it.
2. Otherwise replace the constant with the condition that was meant.

## Examples

### JavaScript

#### Don't

```js
if (0) {
  debugPanel();
}
```

#### Do

```js
if (debug) {
  debugPanel();
}
```

```js
while (true) {
  step();
}
```

### JSX

#### Don't

```jsx
if (0) {
  debugPanel();
}
```

#### Do

```jsx
if (debug) {
  debugPanel();
}
```

```jsx
while (true) {
  step();
}
```

### TypeScript

#### Don't

```ts
if (0) {
  debugPanel();
}
```

#### Do

```ts
if (debug) {
  debugPanel();
}
```

```ts
while (true) {
  step();
}
```

### TSX

#### Don't

```tsx
if (0) {
  debugPanel();
}
```

#### Do

```tsx
if (debug) {
  debugPanel();
}
```

```tsx
while (true) {
  step();
}
```

## Differences from the source

Only literal conditions are checked; ESLint also folds some expressions such as `typeof x` and `!1`.
