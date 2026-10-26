---
id: sec-js/no-secret-in-log
title: "No password, token or secret written to a log"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which fields to keep in the log is a decision for the author"
stability: preview
security: { cwe: [532], owasp: ["A09:2025"], severity: medium }
since: 0.3.0
---

A `console` or logger call is not given a variable or property named like a password, a token, a secret, an API key or an `authorization` header.

**Why:** Logs are copied to files, dashboards and third parties and read by far more people than the secret was meant for (CWE-532).

```codelaw
match:
  all:
    - pattern: $O.$M($$$ARGS)
    - regex: '^(console|logger|log|winston|pino|bunyan)\.(log|info|warn|error|debug|trace|fatal|verbose)\('
    - regex: '(?i)[,({]\s*[\w$.]*(password|passwd|secret|token|api_?key|apikey|authorization|credentials?)[\w$.]*\s*[,)}]|\$\{\s*[\w$.]*(password|passwd|secret|token|apikey|authorization)[\w$.]*\s*\}|:\s*[\w$.]*(password|passwd|secret|token|apikey|authorization)[\w$.]*\s*[,}]'
message: "A secret is written to the log here"
```

## How to fix

1. Leave the value out of the message, or log an id that points to it (`userId`).
2. Redact known fields in the logger (`pino` `redact`, `winston` format) so a new call cannot leak them.

## Examples

### JavaScript

#### Don't

```js
logger.info("signed in", { user, password });
```

#### Do

```js
logger.info("signed in", { user });
```

### JSX

#### Don't

```jsx
logger.info("signed in", { user, password });
```

#### Do

```jsx
logger.info("signed in", { user });
```

### TypeScript

#### Don't

```ts
logger.info("signed in", { user, password });
```

#### Do

```ts
logger.info("signed in", { user });
```

### TSX

#### Don't

```tsx
logger.info("signed in", { user, password });
```

#### Do

```tsx
logger.info("signed in", { user });
```
