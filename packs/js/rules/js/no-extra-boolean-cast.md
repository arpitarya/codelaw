---
id: js/no-extra-boolean-cast
title: "No needless !! or Boolean() in a condition"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: [eslint/no-extra-boolean-cast]
since: 0.1.0
---

A condition (`if`, `while`, `do … while`, `? :`) doesn't convert its value with `!!`.

**Why:** A condition is already evaluated as a boolean, so `!!` only adds noise.

```codelaw
match:
  pattern: "!!$X"
  any:
    - inside:
        stopBy: neighbor
        kind: parenthesized_expression
        inside:
          stopBy: neighbor
          any: [{ kind: if_statement }, { kind: while_statement }, { kind: do_statement }]
          field: condition
    - inside: { stopBy: neighbor, kind: ternary_expression, field: condition }
message: "A condition already converts to boolean; `!!` is redundant"
fix: "$X"
```

## Examples

### JavaScript

#### Don't

```js
if (!!user) {
  greet();
}
```

#### Do

```js
if (user) {
  greet();
}
```

### JSX

#### Don't

```jsx
if (!!user) {
  greet();
}
```

#### Do

```jsx
if (user) {
  greet();
}
```

### TypeScript

#### Don't

```ts
if (!!user) {
  greet();
}
```

#### Do

```ts
if (user) {
  greet();
}
```

### TSX

#### Don't

```tsx
if (!!user) {
  greet();
}
```

#### Do

```tsx
if (user) {
  greet();
}
```

## Differences from the source

`Boolean(x)` in a condition is not checked yet.
