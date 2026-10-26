---
id: sec-js/no-nosql-injection
title: "No request object passed straight to a query, no $where"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which fields the query may filter on is a decision for the author"
stability: preview
security: { cwe: [943], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

A MongoDB-style query is not given `req.body`, `req.query` or `req.params` whole as its filter, and no query object has a `$where` key.

**Why:** A request that sends `{ "$ne": null }` instead of a string turns the filter into an operator expression, and `$where` runs a JavaScript string on the database (CWE-943).

```codelaw
match:
  any:
    - pattern: $O.find($ARG)
    - pattern: $O.find($ARG, $$$REST)
    - pattern: $O.findOne($ARG)
    - pattern: $O.findOne($ARG, $$$REST)
    - pattern: $O.findOneAndUpdate($ARG)
    - pattern: $O.findOneAndUpdate($ARG, $$$REST)
    - pattern: $O.findOneAndDelete($ARG)
    - pattern: $O.findOneAndDelete($ARG, $$$REST)
    - pattern: $O.findOneAndRemove($ARG)
    - pattern: $O.findOneAndRemove($ARG, $$$REST)
    - pattern: $O.updateOne($ARG)
    - pattern: $O.updateOne($ARG, $$$REST)
    - pattern: $O.updateMany($ARG)
    - pattern: $O.updateMany($ARG, $$$REST)
    - pattern: $O.deleteOne($ARG)
    - pattern: $O.deleteOne($ARG, $$$REST)
    - pattern: $O.deleteMany($ARG)
    - pattern: $O.deleteMany($ARG, $$$REST)
    - pattern: $O.replaceOne($ARG)
    - pattern: $O.replaceOne($ARG, $$$REST)
    - pattern: $O.countDocuments($ARG)
    - pattern: $O.countDocuments($ARG, $$$REST)
    - pattern: $O.count($ARG)
    - pattern: $O.count($ARG, $$$REST)
    - pattern: $O.remove($ARG)
    - pattern: $O.remove($ARG, $$$REST)
    - kind: pair
      regex: '^["'']?\$where["'']?\s*:'
constraints:
  ARG: { regex: '^(req|request|ctx\.request)\.(body|query|params)$' }
message: "The query takes request data as its filter; an operator object from the client rewrites it"
```

## How to fix

1. Build the filter from named fields and coerce each to the type the field has: `{ email: String(req.body.email) }`.
2. Validate the body with a schema (zod, joi, ajv) before it reaches the query; reject objects where a string is expected.
3. Replace `$where` with ordinary operators (`$expr`, `$gt`, …); never run a string on the database.

## Examples

### JavaScript

#### Don't

```js
const user = await User.findOne(req.body);
```

#### Do

```js
const user = await User.findOne({ email: String(req.body.email) });
```

### JSX

#### Don't

```jsx
const user = await User.findOne(req.body);
```

#### Do

```jsx
const user = await User.findOne({ email: String(req.body.email) });
```

### TypeScript

#### Don't

```ts
const user = await User.findOne(req.body);
```

#### Do

```ts
const user = await User.findOne({ email: String(req.body.email) });
```

### TSX

#### Don't

```tsx
const user = await User.findOne(req.body);
```

#### Do

```tsx
const user = await User.findOne({ email: String(req.body.email) });
```
