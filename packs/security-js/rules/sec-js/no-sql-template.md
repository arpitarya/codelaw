---
id: sec-js/no-sql-template
title: "No SQL built from a template or concatenation"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the statement and its values have to be separated by hand, and the placeholder style depends on the driver"
stability: preview
ported_from: [bandit/B608, ruff/S608]
security: { cwe: [89], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

A SQL string passed to `query`, `execute`, `raw` and similar calls is a literal with placeholders, not a template literal with `${…}` or a concatenation.

**Why:** A value spliced into SQL changes the statement itself, which is SQL injection (CWE-89); a placeholder sends the value apart from the statement.

```codelaw
match:
  any:
    - pattern: $O.query($SQL)
    - pattern: $O.query($SQL, $$$REST)
    - pattern: $O.raw($SQL)
    - pattern: $O.raw($SQL, $$$REST)
    - pattern: $O.execute($SQL)
    - pattern: $O.execute($SQL, $$$REST)
    - pattern: $O.exec($SQL)
    - pattern: $O.exec($SQL, $$$REST)
    - pattern: $O.all($SQL)
    - pattern: $O.all($SQL, $$$REST)
    - pattern: $O.get($SQL)
    - pattern: $O.get($SQL, $$$REST)
    - pattern: $O.run($SQL)
    - pattern: $O.run($SQL, $$$REST)
    - pattern: $O.prepare($SQL)
    - pattern: $O.prepare($SQL, $$$REST)
    - pattern: $O.each($SQL)
    - pattern: $O.each($SQL, $$$REST)
    - pattern: $O.queryRaw($SQL)
    - pattern: $O.queryRaw($SQL, $$$REST)
constraints:
  SQL:
    any:
      - kind: template_string
        regex: '(?is)\b(select|insert|update|delete|drop|create|alter|truncate|where|union)\b.*\$\{|\$\{.*\b(select|insert|update|delete|drop|create|alter|truncate|where|union)\b'
      - kind: binary_expression
        regex: '(?is)^\s*["''`]\s*(select|insert|update|delete|drop|create|alter|truncate|with)\b'
message: "SQL is built from a value here; send the value as a parameter"
```

## How to fix

1. Write the statement as a literal with placeholders (`$1`, `?` or `:name`, as the driver wants) and pass the values as the second argument.
2. With a query builder (knex, Sequelize, Kysely) use its bindings: `knex.raw("id = ?", [id])`, `replacements`.
3. A table or column name cannot be a placeholder: pick it from an allow-list in code.

## Examples

### JavaScript

#### Don't

```js
const rows = await db.query(`SELECT * FROM users WHERE id = ${id}`);
```

#### Do

```js
const rows = await db.query("SELECT * FROM users WHERE id = $1", [id]);
```

### JSX

#### Don't

```jsx
const rows = await db.query(`SELECT * FROM users WHERE id = ${id}`);
```

#### Do

```jsx
const rows = await db.query("SELECT * FROM users WHERE id = $1", [id]);
```

### TypeScript

#### Don't

```ts
const rows = await db.query(`SELECT * FROM users WHERE id = ${id}`);
```

#### Do

```ts
const rows = await db.query("SELECT * FROM users WHERE id = $1", [id]);
```

### TSX

#### Don't

```tsx
const rows = await db.query(`SELECT * FROM users WHERE id = ${id}`);
```

#### Do

```tsx
const rows = await db.query("SELECT * FROM users WHERE id = $1", [id]);
```
