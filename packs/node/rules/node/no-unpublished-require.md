---
id: node/no-unpublished-require
title: "Published files require only what is published"
languages: [javascript, jsx, typescript, tsx]
scope: project
fix: manual
fix_reason: "whether to publish the file, move the dependency or move the code is the package owner's choice"
stability: preview
ported_from: [eslint-plugin-n/no-unpublished-require]
since: 0.1.0
---

In a package that is not `private` and lists `files`, a file `npm publish` packs requires neither a project file it doesn't pack nor a package that is only a `devDependency`.

**Why:** the published package then fails to load for its users: the file isn't in the tarball, or the devDependency isn't installed.

```codelaw
graph:
  import:
    form: [require]
    published: false
message: "`$SOURCE` is not shipped to this package's users (`$MANIFEST`), but a published file requires it"
```

## How to fix

1. Add the required file to `files` in the nearest `package.json`, move the package from `devDependencies` to `dependencies`, or move the `require()` out of the published code.

## Examples

### JavaScript

#### Don't

```js files
// package.json
{ "name": "lib", "main": "lib/index.js", "files": ["lib"] }
// lib/index.js
const { fixtures } = require("../test/fixtures");
console.log(fixtures);
// test/fixtures.js
export const fixtures = [];
```

#### Do

```js files
// package.json
{ "name": "lib", "main": "lib/index.js", "files": ["lib"] }
// lib/index.js
const { fixtures } = require("./fixtures");
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
const { fixtures } = require("../test/fixtures");
export const App = () => <p>{String(fixtures)}</p>;
// test/fixtures.jsx
export const fixtures = [];
```

#### Do

```jsx files
// package.json
{ "name": "lib", "main": "lib/index.jsx", "files": ["lib"] }
// lib/index.jsx
const { fixtures } = require("./fixtures");
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
const { fixtures } = require("../test/fixtures");
console.log(fixtures);
// test/fixtures.ts
export const fixtures = [];
```

#### Do

```ts files
// package.json
{ "name": "lib", "main": "lib/index.ts", "files": ["lib"] }
// lib/index.ts
const { fixtures } = require("./fixtures");
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
const { fixtures } = require("../test/fixtures");
export const App = () => <p>{String(fixtures)}</p>;
// test/fixtures.tsx
export const fixtures = [];
```

#### Do

```tsx files
// package.json
{ "name": "lib", "main": "lib/index.tsx", "files": ["lib"] }
// lib/index.tsx
const { fixtures } = require("./fixtures");
export const App = () => <p>{String(fixtures)}</p>;
// lib/fixtures.tsx
export const fixtures = [];
```
