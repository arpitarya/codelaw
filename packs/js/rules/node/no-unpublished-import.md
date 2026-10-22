---
id: node/no-unpublished-import
title: "Published files import only what is published"
languages: [javascript, jsx, typescript, tsx]
scope: project
fix: manual
fix_reason: "whether to publish the file, move the dependency or move the code is the package owner's choice"
stability: preview
ported_from: [eslint-plugin-n/no-unpublished-import]
since: 0.1.0
---

In a package that is not `private` and lists `files`, a file `npm publish` packs imports neither a project file it doesn't pack nor a package that is only a `devDependency`.

**Why:** the published package then fails to load for its users: the file isn't in the tarball, or the devDependency isn't installed.

```codelaw
graph:
  import:
    form: [import, export, dynamic]
    published: false
message: "`$SOURCE` is not shipped to this package's users (`$MANIFEST`), but a published file imports it"
```

## How to fix

1. Add the imported file to `files` in the nearest `package.json`, move the package from `devDependencies` to `dependencies`, or move the import out of the published code.
2. A type-only import is not reported: it is erased from the published JavaScript.

## Examples

### JavaScript

#### Don't

```js files
// package.json
{ "name": "lib", "main": "lib/index.js", "files": ["lib"] }
// lib/index.js
import { fixtures } from "../test/fixtures";
console.log(fixtures);
// test/fixtures.js
export const fixtures = [];
```

#### Do

```js files
// package.json
{ "name": "lib", "main": "lib/index.js", "files": ["lib"] }
// lib/index.js
import { fixtures } from "./fixtures";
console.log(fixtures);
// lib/fixtures.js
export const fixtures = [];
```

### JSX

#### Don't

```jsx files
// package.json
{ "name": "lib", "main": "lib/index.jsx", "files": ["lib"] }
// lib/index.jsx
import { fixtures } from "../test/fixtures";
export const App = () => <p>{String(fixtures)}</p>;
// test/fixtures.jsx
export const fixtures = [];
```

#### Do

```jsx files
// package.json
{ "name": "lib", "main": "lib/index.jsx", "files": ["lib"] }
// lib/index.jsx
import { fixtures } from "./fixtures";
export const App = () => <p>{String(fixtures)}</p>;
// lib/fixtures.jsx
export const fixtures = [];
```

### TypeScript

#### Don't

```ts files
// package.json
{ "name": "lib", "main": "lib/index.ts", "files": ["lib"] }
// lib/index.ts
import { fixtures } from "../test/fixtures";
console.log(fixtures);
// test/fixtures.ts
export const fixtures = [];
```

#### Do

```ts files
// package.json
{ "name": "lib", "main": "lib/index.ts", "files": ["lib"] }
// lib/index.ts
import { fixtures } from "./fixtures";
console.log(fixtures);
// lib/fixtures.ts
export const fixtures = [];
```

### TSX

#### Don't

```tsx files
// package.json
{ "name": "lib", "main": "lib/index.tsx", "files": ["lib"] }
// lib/index.tsx
import { fixtures } from "../test/fixtures";
export const App = () => <p>{String(fixtures)}</p>;
// test/fixtures.tsx
export const fixtures = [];
```

#### Do

```tsx files
// package.json
{ "name": "lib", "main": "lib/index.tsx", "files": ["lib"] }
// lib/index.tsx
import { fixtures } from "./fixtures";
export const App = () => <p>{String(fixtures)}</p>;
// lib/fixtures.tsx
export const fixtures = [];
```
