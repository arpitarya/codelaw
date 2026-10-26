---
id: sec-js/no-xpath-injection
title: "No XPath built from a template or concatenation"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "how a value is bound depends on the XPath library"
stability: preview
security: { cwe: [643], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

An XPath expression passed to `xpath.select`, `xpath.evaluate` or `document.evaluate` is a literal, not a template literal or a concatenation.

**Why:** A value in the expression can close the predicate and add its own, so `' or '1'='1` reads every node (CWE-643).

```codelaw
match:
  any:
    - pattern: xpath.select($Q)
    - pattern: xpath.select($Q, $$$REST)
    - pattern: xpath.select1($Q)
    - pattern: xpath.select1($Q, $$$REST)
    - pattern: xpath.evaluate($Q)
    - pattern: xpath.evaluate($Q, $$$REST)
    - pattern: xpath.parse($Q)
    - pattern: xpath.parse($Q, $$$REST)
    - pattern: $D.evaluate($Q)
    - pattern: $D.evaluate($Q, $$$REST)
constraints:
  Q: { any: [ { kind: template_string, regex: '\$\{' }, { kind: binary_expression } ] }
message: "An XPath expression is built from a value here; a value in it can add predicates"
```

## How to fix

1. Pass values as XPath variables (`$name`) where the library supports them, and bind them with its variable resolver.
2. If it cannot, accept only values that match a strict pattern (`^[A-Za-z0-9_-]+$`) before they go into the expression.

## Examples

### JavaScript

#### Don't

```js
const nodes = xpath.select("//user[name='" + name + "']", doc);
```

#### Do

```js
const nodes = xpath.select("//user[name=$name]", doc, false, { name });
```

### JSX

#### Don't

```jsx
const nodes = xpath.select("//user[name='" + name + "']", doc);
```

#### Do

```jsx
const nodes = xpath.select("//user[name=$name]", doc, false, { name });
```

### TypeScript

#### Don't

```ts
const nodes = xpath.select("//user[name='" + name + "']", doc);
```

#### Do

```ts
const nodes = xpath.select("//user[name=$name]", doc, false, { name });
```

### TSX

#### Don't

```tsx
const nodes = xpath.select("//user[name='" + name + "']", doc);
```

#### Do

```tsx
const nodes = xpath.select("//user[name=$name]", doc, false, { name });
```
