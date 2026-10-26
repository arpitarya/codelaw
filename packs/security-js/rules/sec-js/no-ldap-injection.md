---
id: sec-js/no-ldap-injection
title: "No LDAP filter built from a template or concatenation"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which filter classes or escape function the project uses is its own choice"
stability: preview
security: { cwe: [90], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

The `filter` of an LDAP search is a literal or built with the library's filter classes, not a template literal or a concatenation.

**Why:** A value in a filter can add clauses: `*)(uid=*))(|(uid=*` matches every entry, or bypasses a login (CWE-90).

```codelaw
match:
  kind: pair
  regex: '(?s)^["'']?filter["'']?\s*:\s*(`[^`]*\$\{|["''][^"'']*["'']\s*\+|[A-Za-z_$][\w$.]*\s*\+)'
  inside:
    stopBy: end
    pattern: $C.search($$$ARGS)
message: "An LDAP filter is built from a value here; a value in it can add clauses"
```

## How to fix

1. Build the filter with the library's filter objects (ldapjs `EqualityFilter`, `AndFilter`) so values are escaped.
2. Where a string filter is unavoidable, escape each value for the LDAP filter syntax (RFC 4515: `\\`, `*`, `(`, `)` and NUL) before it is spliced in.

## Examples

### JavaScript

#### Don't

```js
client.search(base, { filter: `(uid=${name})`, scope: "sub" }, callback);
```

#### Do

```js
client.search(base, { filter: new EqualityFilter({ attribute: "uid", value: name }), scope: "sub" }, callback);
```

### JSX

#### Don't

```jsx
client.search(base, { filter: `(uid=${name})`, scope: "sub" }, callback);
```

#### Do

```jsx
client.search(base, { filter: new EqualityFilter({ attribute: "uid", value: name }), scope: "sub" }, callback);
```

### TypeScript

#### Don't

```ts
client.search(base, { filter: `(uid=${name})`, scope: "sub" }, callback);
```

#### Do

```ts
client.search(base, { filter: new EqualityFilter({ attribute: "uid", value: name }), scope: "sub" }, callback);
```

### TSX

#### Don't

```tsx
client.search(base, { filter: `(uid=${name})`, scope: "sub" }, callback);
```

#### Do

```tsx
client.search(base, { filter: new EqualityFilter({ attribute: "uid", value: name }), scope: "sub" }, callback);
```
