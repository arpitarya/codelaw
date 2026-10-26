---
id: sec-ts/no-query-builder-raw
title: "No raw SQL fragment from a value in Drizzle, Kysely or MikroORM"
languages: [typescript, tsx]
fix: manual
fix_reason: "the statement and its values have to be separated by hand"
stability: preview
security: { cwe: [89], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

`sql.raw(x)`, `sql.lit(x)` and `em.execute(x)` are not given a value that is not a literal; the `sql` template tag sends each `${…}` as a parameter.

**Why:** These calls put the string into the statement as it is, so a value in it changes the statement (CWE-89).

```codelaw
match:
  any:
    - pattern: sql.raw($X)
    - pattern: sql.lit($X)
    - all:
        - pattern: $EM.execute($X)
        - regex: '^(this\.)?(em|entityManager|manager|orm\.em)\.execute'
    - all:
        - pattern: $EM2.execute($X, $$$R)
        - regex: '^(this\.)?(em|entityManager|manager|orm\.em)\.execute'
constraints:
  X: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
message: "`$X` is put into the SQL statement as it is; a value in it changes the statement"
```

## How to fix

1. Use the `sql` tagged template: each `${value}` becomes a parameter. `sql.raw` is for a fixed fragment you wrote, as a literal.
2. A table or column name that must vary is picked from an allow-list in code, then written with `sql.identifier(name)` (Drizzle) or `sql.id(name)` (Kysely).

## Examples

### TypeScript

#### Don't

```ts
await db.execute(sql.raw(query));
```

#### Do

```ts
await db.execute(sql`select * from users where id = ${id}`);
```

### TSX

#### Don't

```tsx
await db.execute(sql.raw(query));
```

#### Do

```tsx
await db.execute(sql`select * from users where id = ${id}`);
```
