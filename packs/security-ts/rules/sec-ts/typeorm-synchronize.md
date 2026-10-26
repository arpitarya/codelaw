---
id: sec-ts/typeorm-synchronize
title: "TypeORM does not synchronize the schema on start"
languages: [typescript, tsx]
fix: manual
fix_reason: "how the schema is changed instead (migrations) is a decision for the author"
stability: preview
security: { cwe: [1188], owasp: ["A02:2025"], severity: medium }
since: 0.3.0
---

A DataSource or `TypeOrmModule` configuration does not set `synchronize: true`.

**Why:** `synchronize` rewrites the database to match the entities on every start: a renamed field drops a column and its data, and it runs against whatever database the app is pointed at (CWE-1188).

```codelaw
match:
  pattern:
    context: '({ synchronize: true })'
    selector: pair
message: "`synchronize: true` changes the database schema on every start"
```

## How to fix

1. Set `synchronize: false` (the default) and write migrations: `typeorm migration:generate`, then `migration:run` at deploy.
2. If it is only for a local database, set it from an environment variable that is never true in production.

## Examples

### TypeScript

#### Don't

```ts
export const db = new DataSource({ type: "postgres", url, synchronize: true });
```

#### Do

```ts
export const db = new DataSource({ type: "postgres", url, migrations: ["dist/migrations/*.js"] });
```

### TSX

#### Don't

```tsx
export const db = new DataSource({ type: "postgres", url, synchronize: true });
```

#### Do

```tsx
export const db = new DataSource({ type: "postgres", url, migrations: ["dist/migrations/*.js"] });
```
