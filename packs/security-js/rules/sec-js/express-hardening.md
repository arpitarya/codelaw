---
id: sec-js/express-hardening
title: "An Express app turns off x-powered-by or uses helmet"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "`app.use(helmet())` is the better fix and needs an import; the inserted line only hides the framework"
stability: preview
security: { cwe: [200], owasp: ["A02:2025"], severity: low }
since: 0.3.0
---

A file that creates an Express app also calls `helmet()` or disables `x-powered-by`.

**Why:** The default `X-Powered-By: Express` header names the framework for every scanner, and without `helmet` the app sends none of the headers that limit framing, sniffing and mixed content (CWE-200).

```codelaw
match:
  all:
    - pattern: const $A = express()
    - not:
        inside:
          kind: program
          stopBy: end
          has:
            stopBy: end
            regex: 'helmet|x-powered-by'
message: "This Express app does not call `helmet()` or disable `x-powered-by`"
fix:
  - insert-after: "\n$A.disable(\"x-powered-by\");"
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
const app = express();
app.get("/", handler);
```

#### Do

```js
const app = express();
app.use(helmet());
app.get("/", handler);
```

#### Fixed

```js
const app = express();
app.disable("x-powered-by");
app.get("/", handler);
```

### JSX

#### Don't

```jsx
const app = express();
app.get("/", handler);
```

#### Do

```jsx
const app = express();
app.use(helmet());
app.get("/", handler);
```

#### Fixed

```jsx
const app = express();
app.disable("x-powered-by");
app.get("/", handler);
```

### TypeScript

#### Don't

```ts
const app = express();
app.get("/", handler);
```

#### Do

```ts
const app = express();
app.use(helmet());
app.get("/", handler);
```

#### Fixed

```ts
const app = express();
app.disable("x-powered-by");
app.get("/", handler);
```

### TSX

#### Don't

```tsx
const app = express();
app.get("/", handler);
```

#### Do

```tsx
const app = express();
app.use(helmet());
app.get("/", handler);
```

#### Fixed

```tsx
const app = express();
app.disable("x-powered-by");
app.get("/", handler);
```
