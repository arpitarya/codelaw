---
id: sec-ts/nest-cors
title: "NestJS CORS names its origins"
languages: [typescript, tsx]
fix: manual
fix_reason: "which origins may call the API is a decision for the author"
stability: preview
security: { cwe: [942], owasp: ["A02:2025"], severity: high }
since: 0.3.0
---

`app.enableCors()` is given options with an origin list, not no options, and not `origin: true` together with `credentials: true`; `NestFactory.create(…, { cors: true })` is not used.

**Why:** `enableCors()` allows every origin, and `origin: true` reflects the caller's; with credentials any website can use the visitor's session on your API (CWE-942).

```codelaw
match:
  any:
    - pattern: $A.enableCors()
    - all:
        - pattern: $A2.enableCors($OPTS)
        - has:
            kind: pair
            stopBy: end
            regex: '^["'']?origin["'']?\s*:\s*(["'']\*["'']|true)\s*$'
        - has:
            kind: pair
            stopBy: end
            regex: '^["'']?credentials["'']?\s*:\s*true\s*$'
    - pattern:
        context: '({ cors: true })'
        selector: pair
message: "CORS allows every origin here"
```

## How to fix

1. Pass the origins that may call the API: `app.enableCors({ origin: ["https://app.example.com"] })`.
2. Keep `credentials: true` only with an explicit origin list.

## Examples

### TypeScript

#### Don't

```ts
const app = await NestFactory.create(AppModule);
app.enableCors();
```

#### Do

```ts
const app = await NestFactory.create(AppModule);
app.enableCors({ origin: ["https://app.example.com"], credentials: true });
```

### TSX

#### Don't

```tsx
const app = await NestFactory.create(AppModule);
app.enableCors();
```

#### Do

```tsx
const app = await NestFactory.create(AppModule);
app.enableCors({ origin: ["https://app.example.com"], credentials: true });
```
