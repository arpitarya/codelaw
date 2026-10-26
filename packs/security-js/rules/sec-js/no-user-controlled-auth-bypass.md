---
id: sec-js/no-user-controlled-auth-bypass
title: "No privileged branch decided by a request value"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "where the decision should come from is a design choice"
stability: preview
security: { cwe: [807], owasp: ["A01:2025"], severity: high }
since: 0.3.0
---

An `if` that guards a privileged action does not test a cookie, a query value, a header or a body field named `isAdmin`, `role`, `debug`, `bypass` or similar.

**Why:** The client writes every one of those; whoever sets the flag gets the branch (CWE-807).

```codelaw
match:
  kind: if_statement
  has:
    field: condition
    regex: '(?i)(req|request)\.(cookies|query|body|headers|params)(\.|\[\s*["''])\s*(is_?admin|admin|is_?staff|superuser|is_?superuser|role|debug|bypass|skip_?auth|authorized|authenticated|internal|test_?mode|x-admin)'
message: "A branch is chosen by a value the client writes"
```

## How to fix

1. Decide from server-side state: the session, a verified token, a database row.
2. Remove debug and bypass switches from request handling; read them from the environment at start-up instead.

## Examples

### JavaScript

#### Don't

```js
if (req.cookies.isAdmin === "true") {
  showAdminPanel(res);
}
```

#### Do

```js
if (req.session.user && req.session.user.role === "admin") {
  showAdminPanel(res);
}
```

### JSX

#### Don't

```jsx
if (req.cookies.isAdmin === "true") {
  showAdminPanel(res);
}
```

#### Do

```jsx
if (req.session.user && req.session.user.role === "admin") {
  showAdminPanel(res);
}
```

### TypeScript

#### Don't

```ts
if (req.cookies.isAdmin === "true") {
  showAdminPanel(res);
}
```

#### Do

```ts
if (req.session.user && req.session.user.role === "admin") {
  showAdminPanel(res);
}
```

### TSX

#### Don't

```tsx
if (req.cookies.isAdmin === "true") {
  showAdminPanel(res);
}
```

#### Do

```tsx
if (req.session.user && req.session.user.role === "admin") {
  showAdminPanel(res);
}
```
