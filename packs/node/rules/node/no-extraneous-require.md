---
id: node/no-extraneous-require
title: "Required packages must be declared"
languages: [javascript, jsx, typescript, tsx]
scope: project
fix: manual
fix_reason: "which dependency field and version to add is the package owner's choice"
stability: preview
ported_from: [eslint-plugin-n/no-extraneous-require]
since: 0.1.0
---

Every package a `require()` names is declared in the nearest `package.json`: `dependencies`, `devDependencies`, `peerDependencies` or `optionalDependencies`.

**Why:** an undeclared package works only while something else happens to install it, and breaks for every user who doesn't have it.

```codelaw
graph:
  import:
    form: [require]
    declared: false
message: "`$PACKAGE` is required but not declared in `$MANIFEST`"
```

## How to fix

1. Add the package to the nearest `package.json` (`dependencies` if the shipped code needs it, `devDependencies` if only tests and tooling do), with your package manager's `add` command.

## Examples

### JavaScript

#### Don't

```js files
// package.json
{ "name": "app", "dependencies": { "react": "19.0.0" } }
// src/app.js
const lodash = require("lodash");
console.log(lodash);
```

#### Do

```js files
// package.json
{ "name": "app", "dependencies": { "react": "19.0.0", "lodash": "4.17.21" } }
// src/app.js
const lodash = require("lodash");
console.log(lodash);
```

### JSX

#### Don't

```jsx files
// package.json
{ "name": "app", "dependencies": { "react": "19.0.0" } }
// src/app.jsx
const lodash = require("lodash");
export const App = () => <p>{String(lodash)}</p>;
```

#### Do

```jsx files
// package.json
{ "name": "app", "dependencies": { "react": "19.0.0", "lodash": "4.17.21" } }
// src/app.jsx
const lodash = require("lodash");
export const App = () => <p>{String(lodash)}</p>;
```

### TypeScript

#### Don't

```ts files
// package.json
{ "name": "app", "dependencies": { "react": "19.0.0" } }
// src/app.ts
const lodash = require("lodash");
console.log(lodash);
```

#### Do

```ts files
// package.json
{ "name": "app", "dependencies": { "react": "19.0.0", "lodash": "4.17.21" } }
// src/app.ts
const lodash = require("lodash");
console.log(lodash);
```

### TSX

#### Don't

```tsx files
// package.json
{ "name": "app", "dependencies": { "react": "19.0.0" } }
// src/app.tsx
const lodash = require("lodash");
export const App = () => <p>{String(lodash)}</p>;
```

#### Do

```tsx files
// package.json
{ "name": "app", "dependencies": { "react": "19.0.0", "lodash": "4.17.21" } }
// src/app.tsx
const lodash = require("lodash");
export const App = () => <p>{String(lodash)}</p>;
```
