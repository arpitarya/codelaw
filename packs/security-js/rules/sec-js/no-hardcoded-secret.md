---
id: sec-js/no-hardcoded-secret
title: "No secret written into the source"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "where the secret comes from instead is a deployment decision"
stability: preview
ported_from: [bandit/B105, ruff/S105]
security: { cwe: [798], owasp: ["A07:2025"], severity: high }
since: 0.3.0
---

A cloud key, a token, a private key or a password is not a string literal in the code.

**Why:** Source goes to every clone, every CI log and every build; a secret in it is as public as the repository will ever be, and removing it later does not unpublish it (CWE-798).

```codelaw
match:
  any:
    - kind: string
      regex: '^["'']?(AKIA|ASIA)[0-9A-Z]{16}["'']?$|AIza[0-9A-Za-z_-]{35}|\bgh[pousr]_[A-Za-z0-9]{36,}|\bgithub_pat_[A-Za-z0-9_]{50,}|\b[sr]k_(live|test)_[0-9a-zA-Z]{24,}|\bxox[abprs]-[0-9A-Za-z-]{10,}|-----BEGIN ([A-Z]+ )?PRIVATE KEY|\bsk-(ant-)?[A-Za-z0-9_-]{32,}'
    - kind: template_string
      regex: '-----BEGIN ([A-Z]+ )?PRIVATE KEY'
    - all:
        - kind: variable_declarator
        - regex: '(?i)^[\w$]*(password|passwd|pwd|secret|api_?key|apikey|auth_?token|access_?token|private_?key|client_?secret)[\w$]*\s*=\s*["''][^"'']{8,}["'']$'
        - not:
            regex: '(?i)^[\w$]*(name|header|field|cookie|label|prefix|url|path|endpoint)[\w$]*\s*='
    - all:
        - kind: assignment_expression
        - regex: '(?i)^[\w$.]*(password|passwd|pwd|secret|api_?key|apikey|auth_?token|access_?token|private_?key|client_?secret)[\w$]*\s*=\s*["''][^"'']{8,}["'']$'
        - not:
            regex: '(?i)^[\w$.]*(name|header|field|cookie|label|prefix|url|path|endpoint)[\w$]*\s*='
    - all:
        - kind: pair
        - regex: '(?i)^["'']?[\w$]*(password|passwd|pwd|secret|api_?key|apikey|auth_?token|access_?token|private_?key|client_?secret)[\w$]*["'']?\s*:\s*["''][^"'']{8,}["'']$'
        - not:
            regex: '(?i)^["'']?[\w$]*(name|header|field|cookie|label|prefix|url|path|endpoint)[\w$]*["'']?\s*:'
message: "A secret is written into the source here"
```

## How to fix

1. Read the value from the environment or a secret store (`process.env.NAME`, AWS Secrets Manager, Vault) at start-up.
2. Treat the committed value as leaked: revoke it, issue a new one and, if it must be gone from history, rewrite it.
3. Keep test keys in a clearly fake form (`test_key_000`) so the scanner can tell them apart.

## Examples

### JavaScript

#### Don't

```js
const awsAccessKey = "AKIAIOSFODNN7EXAMPLE";
const dbPassword = "correct-horse-battery";
```

#### Do

```js
const awsAccessKey = process.env.AWS_ACCESS_KEY_ID;
const dbPassword = process.env.DB_PASSWORD;
```

### JSX

#### Don't

```jsx
const awsAccessKey = "AKIAIOSFODNN7EXAMPLE";
const dbPassword = "correct-horse-battery";
```

#### Do

```jsx
const awsAccessKey = process.env.AWS_ACCESS_KEY_ID;
const dbPassword = process.env.DB_PASSWORD;
```

### TypeScript

#### Don't

```ts
const awsAccessKey = "AKIAIOSFODNN7EXAMPLE";
const dbPassword = "correct-horse-battery";
```

#### Do

```ts
const awsAccessKey = process.env.AWS_ACCESS_KEY_ID;
const dbPassword = process.env.DB_PASSWORD;
```

### TSX

#### Don't

```tsx
const awsAccessKey = "AKIAIOSFODNN7EXAMPLE";
const dbPassword = "correct-horse-battery";
```

#### Do

```tsx
const awsAccessKey = process.env.AWS_ACCESS_KEY_ID;
const dbPassword = process.env.DB_PASSWORD;
```
