---
id: sec-js/no-non-literal-require
title: "No require or import of a computed module name"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which modules may be loaded is a decision for the author"
stability: preview
ported_from: [eslint-plugin-security/detect-non-literal-require, eslint-plugin-n/no-dynamic-require]
security: { cwe: [829, 95], owasp: ["A08:2025"], severity: medium }
since: 0.3.0
---

`require(x)` and `import(x)` name their module with a literal.

**Why:** A module name built from a value lets that value choose which file runs, so an attacker who reaches it runs code from a path they pick (CWE-829, CWE-95).

```codelaw
match:
  any:
    - pattern: require($X)
    - pattern: import($X)
constraints:
  X: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
message: "The module name here is not a literal; whatever sets `$X` chooses the code that runs"
```

## How to fix

1. Name the module with a literal, or pick it from a table of literals: `const plugins = { csv: () => import("./csv.js") }`.
2. If the name must come from configuration, check it against an allow-list before it reaches `require`.

## Examples

### JavaScript

#### Don't

```js
const plugin = require(pluginName);
```

#### Do

```js
const plugin = require("./plugins/default");
```

### JSX

#### Don't

```jsx
const plugin = require(pluginName);
```

#### Do

```jsx
const plugin = require("./plugins/default");
```

### TypeScript

#### Don't

```ts
const plugin = require(pluginName);
```

#### Do

```ts
const plugin = require("./plugins/default");
```

### TSX

#### Don't

```tsx
const plugin = require(pluginName);
```

#### Do

```tsx
const plugin = require("./plugins/default");
```
