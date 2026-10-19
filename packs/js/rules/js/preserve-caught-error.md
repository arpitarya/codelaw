---
id: js/preserve-caught-error
title: "A rethrown error keeps the caught one as its cause"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "the new error class must accept an options argument"
stability: preview
ported_from: [eslint/preserve-caught-error]
since: 0.1.0
---

A built-in error (`Error`, `TypeError` and the rest) thrown from a `catch` block passes the caught
error as `{ cause }`.

**Why:** Without the cause, the original error and its stack are lost.

```codelaw
match:
  pattern: throw new $C($MSG);
  inside:
    kind: catch_clause
    has: { field: parameter, pattern: $E }
    stopBy:
      any: [{kind: function_declaration}, {kind: function_expression}, {kind: arrow_function}, {kind: method_definition}, {kind: generator_function_declaration}, {kind: generator_function}]
constraints:
  C: { regex: ^(Error|TypeError|RangeError|ReferenceError|SyntaxError|URIError|EvalError)$ }
  MSG: { not: { kind: object } }
where:
  $C: builtin
message: "Pass the caught error as the cause: `new $C($MSG, { cause: $E })`"
fix: "throw new $C($MSG, { cause: $E });"
```

## Examples

### JavaScript

#### Don't

```js
try {
  load();
} catch (err) {
  throw new Error("Load failed");
}
```

#### Do

```js
try {
  load();
} catch (err) {
  throw new Error("Load failed", { cause: err });
}
```

### JSX

#### Don't

```jsx
try {
  load();
} catch (err) {
  throw new Error("Load failed");
}
```

#### Do

```jsx
try {
  load();
} catch (err) {
  throw new Error("Load failed", { cause: err });
}
```

### TypeScript

#### Don't

```ts
try {
  load();
} catch (err) {
  throw new Error("Load failed");
}
```

#### Do

```ts
try {
  load();
} catch (err) {
  throw new Error("Load failed", { cause: err });
}
```

### TSX

#### Don't

```tsx
try {
  load();
} catch (err) {
  throw new Error("Load failed");
}
```

#### Do

```tsx
try {
  load();
} catch (err) {
  throw new Error("Load failed", { cause: err });
}
```
