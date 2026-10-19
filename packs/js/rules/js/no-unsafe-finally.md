---
id: js/no-unsafe-finally
title: "No return, throw, break or continue in finally"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what should happen to the try block's result is a decision"
stability: preview
ported_from: [eslint/no-unsafe-finally]
since: 0.1.0
---

A `finally` block doesn't end the function or loop with `return`, `throw`, `break` or `continue`.

**Why:** Control flow in `finally` silently replaces whatever the `try` or `catch` block returned or
threw.

```codelaw
match:
  any:
    - kind: return_statement
    - kind: throw_statement
    - kind: break_statement
    - kind: continue_statement
  inside:
    kind: finally_clause
    stopBy:
      any: [{kind: function_declaration}, {kind: function_expression}, {kind: arrow_function}, {kind: method_definition}, {kind: generator_function_declaration}, {kind: generator_function}, { kind: class_declaration }]
  not:
    all:
      - any: [{ kind: break_statement }, { kind: continue_statement }]
      - inside:
          stopBy: { kind: finally_clause }
          any: [{ kind: for_statement }, { kind: for_in_statement }, { kind: while_statement }, { kind: do_statement }, { kind: switch_statement }]
message: "Control flow in `finally` overrides the try block's return or exception"
```

## How to fix

1. Move the `return` or `throw` out of `finally` into the `try` or `catch` block.

## Examples

### JavaScript

#### Don't

```js
function load() {
  try {
    return fetchAll();
  } finally {
    return null;
  }
}
```

#### Do

```js
function load() {
  try {
    return fetchAll();
  } finally {
    cleanup();
  }
}
```

### JSX

#### Don't

```jsx
function load() {
  try {
    return fetchAll();
  } finally {
    return null;
  }
}
```

#### Do

```jsx
function load() {
  try {
    return fetchAll();
  } finally {
    cleanup();
  }
}
```

### TypeScript

#### Don't

```ts
function load() {
  try {
    return fetchAll();
  } finally {
    return null;
  }
}
```

#### Do

```ts
function load() {
  try {
    return fetchAll();
  } finally {
    cleanup();
  }
}
```

### TSX

#### Don't

```tsx
function load() {
  try {
    return fetchAll();
  } finally {
    return null;
  }
}
```

#### Do

```tsx
function load() {
  try {
    return fetchAll();
  } finally {
    cleanup();
  }
}
```
