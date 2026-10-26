---
id: sec-ts/nest-validation-whitelist
title: "ValidationPipe has whitelist on"
languages: [typescript, tsx]
fix: suggest
fix_reason: "a client that sends fields no DTO names starts getting them stripped, which can change what an endpoint does"
stability: preview
security: { cwe: [915], owasp: ["A01:2025"], severity: high }
since: 0.3.0
---

`new ValidationPipe(…)` is given `whitelist: true`, so a property no DTO decorator names is stripped.

**Why:** Without it a request can carry extra fields (`isAdmin`, `role`) that the DTO never declared and the code copies into a model: mass assignment (CWE-915).

```codelaw
match:
  any:
    - pattern: new $C()
    - all:
        - pattern: new ValidationPipe($OPTS)
        - not:
            regex: 'whitelist'
constraints:
  C: { regex: '^ValidationPipe$' }
message: "`ValidationPipe` is not told to strip fields the DTO does not name"
fix:
  - replace: "new $C({ whitelist: true })"
```

## How to fix


## Examples

### TypeScript

#### Don't

```ts
app.useGlobalPipes(new ValidationPipe());
```

#### Do

```ts
app.useGlobalPipes(new ValidationPipe({ whitelist: true }));
```

#### Fixed

```ts
app.useGlobalPipes(new ValidationPipe({ whitelist: true }));
```

### TSX

#### Don't

```tsx
app.useGlobalPipes(new ValidationPipe());
```

#### Do

```tsx
app.useGlobalPipes(new ValidationPipe({ whitelist: true }));
```

#### Fixed

```tsx
app.useGlobalPipes(new ValidationPipe({ whitelist: true }));
```
