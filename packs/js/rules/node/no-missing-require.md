---
id: node/no-missing-require
title: "Required project paths must exist"
languages: [javascript, jsx, typescript, tsx]
scope: project
fix: manual
fix_reason: "the right path depends on where the module went"
stability: preview
ported_from: [eslint-plugin-n/no-missing-require]
since: 0.1.0
---

A `require()` path of the project — relative, absolute or a tsconfig `paths` alias — names a file that exists.

**Why:** a path that names no file throws `MODULE_NOT_FOUND`, and only when that line runs.

```codelaw
graph:
  import:
    form: [require]
    resolves: false
message: "`$SOURCE` names no file of this project"
```

## How to fix

1. Point the specifier at the file that exists (check the extension and the folder), or restore the missing file.

## Examples

### JavaScript

#### Don't

```js files
// src/app.js
const { helper } = require("./helpers");
console.log(helper);
// src/helper.js
export const helper = 1;
```

#### Do

```js files
// src/app.js
const { helper } = require("./helper");
console.log(helper);
// src/helper.js
export const helper = 1;
```

### JSX

#### Don't

```jsx files
// src/app.jsx
const { helper } = require("./helpers");
export const App = () => <p>{String(helper)}</p>;
// src/helper.jsx
export const helper = 1;
```

#### Do

```jsx files
// src/app.jsx
const { helper } = require("./helper");
export const App = () => <p>{String(helper)}</p>;
// src/helper.jsx
export const helper = 1;
```

### TypeScript

#### Don't

```ts files
// src/app.ts
const { helper } = require("./helpers");
console.log(helper);
// src/helper.ts
export const helper = 1;
```

#### Do

```ts files
// src/app.ts
const { helper } = require("./helper");
console.log(helper);
// src/helper.ts
export const helper = 1;
```

### TSX

#### Don't

```tsx files
// src/app.tsx
const { helper } = require("./helpers");
export const App = () => <p>{String(helper)}</p>;
// src/helper.tsx
export const helper = 1;
```

#### Do

```tsx files
// src/app.tsx
const { helper } = require("./helper");
export const App = () => <p>{String(helper)}</p>;
// src/helper.tsx
export const helper = 1;
```
