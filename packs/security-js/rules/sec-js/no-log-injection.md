---
id: sec-js/no-log-injection
title: "No request data written to a log unchanged"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "where the value goes in the log line is the author's choice"
stability: preview
security: { cwe: [117], owasp: ["A09:2025"], severity: medium }
since: 0.3.0
---

A log call is not given request data as its message.

**Why:** A value with a newline in it writes a line the attacker composed, so a forged entry sits beside the real ones (CWE-117).

```codelaw
match:
  any:
    - pattern: console.log($V)
    - pattern: console.log($V, $$$REST)
    - pattern: console.info($V)
    - pattern: console.info($V, $$$REST)
    - pattern: console.warn($V)
    - pattern: console.warn($V, $$$REST)
    - pattern: console.error($V)
    - pattern: console.error($V, $$$REST)
    - pattern: console.debug($V)
    - pattern: console.debug($V, $$$REST)
    - pattern: logger.info($V)
    - pattern: logger.info($V, $$$REST)
    - pattern: logger.warn($V)
    - pattern: logger.warn($V, $$$REST)
    - pattern: logger.error($V)
    - pattern: logger.error($V, $$$REST)
    - pattern: logger.debug($V)
    - pattern: logger.debug($V, $$$REST)
    - pattern: logger.log($V)
    - pattern: logger.log($V, $$$REST)
    - pattern: log.info($V)
    - pattern: log.info($V, $$$REST)
    - pattern: log.warn($V)
    - pattern: log.warn($V, $$$REST)
    - pattern: log.error($V)
    - pattern: log.error($V, $$$REST)
    - pattern: log.debug($V)
    - pattern: log.debug($V, $$$REST)
where:
  $V: { from_source: { tables: [sources/express, sources/koa, sources/nextjs] } }
message: "`$V` comes from the request and is written to the log as it is"
```

## How to fix

1. Log the value as a separate field of a structured log (`logger.info({ user }, "login failed")`), which the logger escapes.
2. Or strip line breaks first: `value.replace(/[\r\n]/g, "")`, or `JSON.stringify(value)`.

## Examples

### JavaScript

#### Don't

```js
logger.info(`login failed for ${req.body.user}`);
```

#### Do

```js
logger.info("login failed for %s", JSON.stringify(req.body.user));
```

### JSX

#### Don't

```jsx
logger.info(`login failed for ${req.body.user}`);
```

#### Do

```jsx
logger.info("login failed for %s", JSON.stringify(req.body.user));
```

### TypeScript

#### Don't

```ts
logger.info(`login failed for ${req.body.user}`);
```

#### Do

```ts
logger.info("login failed for %s", JSON.stringify(req.body.user));
```

### TSX

#### Don't

```tsx
logger.info(`login failed for ${req.body.user}`);
```

#### Do

```tsx
logger.info("login failed for %s", JSON.stringify(req.body.user));
```
