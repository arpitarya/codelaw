---
id: sec-ts/no-prisma-raw-unsafe
title: "No $queryRawUnsafe or $executeRawUnsafe with a value that is not a literal"
languages: [typescript, tsx]
fix: suggest
fix_reason: "a `${…}` in the template becomes a bound parameter, which a statement that used it for a table or column name cannot do; only `$queryRawUnsafe` with one template is rewritten"
stability: preview
security: { cwe: [89], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

Prisma's `$queryRawUnsafe` and `$executeRawUnsafe` are not given a SQL string that is built or not a literal; the tagged `$queryRaw` and `$executeRaw` are used.

**Why:** The `Unsafe` calls take a plain string, so a value spliced into it changes the statement (CWE-89); the tagged templates send each `${…}` as a parameter.

```codelaw
match:
  any:
    - all:
        - pattern: $P.$queryRawUnsafe($T)
        - regex: '\$queryRawUnsafe\(\s*`[^`]*`\s*\)$'
    - all:
        - pattern: $P2.$M($Q, $$$R)
        - regex: '\$(queryRaw|executeRaw)Unsafe\('
    - all:
        - pattern: $P3.$M3($Q3)
        - regex: '\$(queryRaw|executeRaw)Unsafe\('
constraints:
  Q: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
  Q3: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
message: "SQL is passed to Prisma as a plain string; a value in it changes the statement"
fix:
  - replace: "$P.$queryRaw$T"
```

## How to fix


## Examples

### TypeScript

#### Don't

```ts
const rows = await prisma.$queryRawUnsafe(`SELECT * FROM "User" WHERE id = ${id}`);
```

#### Do

```ts
const rows = await prisma.$queryRaw`SELECT * FROM "User" WHERE id = ${id}`;
```

#### Fixed

```ts
const rows = await prisma.$queryRaw`SELECT * FROM "User" WHERE id = ${id}`;
```

### TSX

#### Don't

```tsx
const rows = await prisma.$queryRawUnsafe(`SELECT * FROM "User" WHERE id = ${id}`);
```

#### Do

```tsx
const rows = await prisma.$queryRaw`SELECT * FROM "User" WHERE id = ${id}`;
```

#### Fixed

```tsx
const rows = await prisma.$queryRaw`SELECT * FROM "User" WHERE id = ${id}`;
```
