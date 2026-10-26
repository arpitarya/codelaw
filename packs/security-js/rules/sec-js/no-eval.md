---
id: sec-js/no-eval
title: "No eval, new Function or string timers"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what the string stood for (a function, a table of operations, a parser) is a design choice"
stability: preview
ported_from: [eslint-plugin-security/detect-eval-with-expression, bandit/B307, ruff/S307]
security: { cwe: [94], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

Code is not built from a string at run time: no `eval`, `new Function`, `Function(…)`, and no `setTimeout` or `setInterval` given a string.

**Why:** Whatever reaches the string runs with the program's full privileges, so a single untrusted byte in it is remote code execution (CWE-94).

```codelaw
match:
  any:
    - pattern: eval($$$ARGS)
    - pattern: new Function($$$ARGS)
    - pattern: Function($$$ARGS)
    - pattern: setTimeout($CODE)
    - pattern: setTimeout($CODE, $$$REST)
    - pattern: setInterval($CODE)
    - pattern: setInterval($CODE, $$$REST)
constraints:
  CODE: { any: [ { kind: string }, { kind: template_string } ] }
message: "Code is built from a string here and run with the program's privileges"
```

## How to fix

1. Pass a function, not a string, to `setTimeout` and `setInterval`.
2. Parse data with `JSON.parse`; for an expression language use a lookup table of allowed operations or a real parser instead of `eval`.
3. Replace `new Function(…)` with a named function that takes its inputs as parameters.

## Examples

### JavaScript

#### Don't

```js
const result = eval(expression);
setTimeout("tick()", 100);
```

#### Do

```js
const result = JSON.parse(payload);
setTimeout(() => tick(), 100);
```

### JSX

#### Don't

```jsx
const result = eval(expression);
setTimeout("tick()", 100);
```

#### Do

```jsx
const result = JSON.parse(payload);
setTimeout(() => tick(), 100);
```

### TypeScript

#### Don't

```ts
const result = eval(expression);
setTimeout("tick()", 100);
```

#### Do

```ts
const result = JSON.parse(payload);
setTimeout(() => tick(), 100);
```

### TSX

#### Don't

```tsx
const result = eval(expression);
setTimeout("tick()", 100);
```

#### Do

```tsx
const result = JSON.parse(payload);
setTimeout(() => tick(), 100);
```
