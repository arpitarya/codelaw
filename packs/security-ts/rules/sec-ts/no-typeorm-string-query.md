---
id: sec-ts/no-typeorm-string-query
title: "No TypeORM query built from a template or concatenation"
languages: [typescript, tsx]
fix: manual
fix_reason: "the condition and its values have to be separated by hand"
stability: preview
ported_from: [bandit/B608, ruff/S608]
security: { cwe: [89], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

TypeORM's `.query(…)` and the query builder's `.where`, `.andWhere`, `.orWhere` and `.having` are not given a template literal with `${…}` or a concatenation; named parameters are used.

**Why:** A value spliced into the string changes the statement (CWE-89); `:name` placeholders with a parameters object are sent apart from it.

```codelaw
match:
  any:
    - pattern: $O.query($SQL)
    - pattern: $O.query($SQL, $$$REST)
    - pattern: $O.where($SQL)
    - pattern: $O.where($SQL, $$$REST)
    - pattern: $O.andWhere($SQL)
    - pattern: $O.andWhere($SQL, $$$REST)
    - pattern: $O.orWhere($SQL)
    - pattern: $O.orWhere($SQL, $$$REST)
    - pattern: $O.having($SQL)
    - pattern: $O.having($SQL, $$$REST)
constraints:
  SQL:
    any:
      - kind: template_string
        regex: '\$\{'
      - kind: binary_expression
        regex: '^\s*["''`].*\+|\+\s*["''`]'
message: "A TypeORM query is built from a value here; send the value as a parameter"
```

## How to fix

1. Write the condition with a named parameter (`"u.id = :id"`) and pass the values as the second argument: `.where("u.id = :id", { id })`.
2. For `repo.query`, use the driver's placeholders (`$1`, `?`) and an array of values.
3. A column or table name cannot be a parameter: take it from an allow-list in code.

## Examples

### TypeScript

#### Don't

```ts
const user = await repo.createQueryBuilder("u").where(`u.id = ${id}`).getOne();
```

#### Do

```ts
const user = await repo.createQueryBuilder("u").where("u.id = :id", { id }).getOne();
```

### TSX

#### Don't

```tsx
const user = await repo.createQueryBuilder("u").where(`u.id = ${id}`).getOne();
```

#### Do

```tsx
const user = await repo.createQueryBuilder("u").where("u.id = :id", { id }).getOne();
```
