---
id: js/no-undef
title: "No references to undeclared names"
languages: [javascript, jsx]
fix: manual
fix_reason: "the right declaration — an import, a local, or an environment — is the author's call"
stability: preview
options:
  env: { type: "string[]", default: ["shared-node-browser"], description: "environments whose globals are declared: browser, node, commonjs, worker, serviceworker, shared-node-browser, jest, mocha, vitest" }
  globals: { type: "string[]", default: [], description: "more names to treat as declared" }
  typeof: { type: "boolean", default: false, description: "also report names used only as `typeof x`" }
ported_from: [eslint/no-undef]
since: 0.1.0
---

Every name the code reads or writes is declared in the file, built into JavaScript, or a global of
an environment the rule's `env` option names.

**Why:** An undeclared name is usually a typo or a missing import, and throws a ReferenceError when
that line runs.

```codelaw
native: js/no-undef
```

## How to fix

1. If it's a typo, correct the name.
2. If it comes from another module, import it.
3. If it's a global of the runtime (browser, Node), add that environment to the rule's `env` option.

## Examples

### JavaScript

#### Don't

```js
const total = prices.reduce(add, 0);
function add(a, b) {
  return a + b;
}
```

#### Do

```js
const prices = [1, 2];
const total = prices.reduce(add, 0);
function add(a, b) {
  return a + b;
}
```

### JSX

#### Don't

```jsx
const total = prices.reduce(add, 0);
function add(a, b) {
  return a + b;
}
```

#### Do

```jsx
const prices = [1, 2];
const total = prices.reduce(add, 0);
function add(a, b) {
  return a + b;
}
```
