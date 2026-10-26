---
id: sec-ts/next-server-action-validation
title: "A Next.js server action validates its input"
languages: [typescript, tsx]
fix: manual
fix_reason: "the schema and the authorisation check are written by the author"
stability: preview
security: { cwe: [20, 862], owasp: ["A01:2025"], severity: high }
since: 0.3.0
---

An exported function in a `"use server"` file that takes arguments calls a validator (`schema.parse`, `safeParse`, `validate`) on them.

**Why:** A server action is a public HTTP endpoint: its arguments come from the client whatever the form looks like, so an unchecked one writes what an attacker sends (CWE-20, CWE-862). Whether the function also checks who is calling is not decided from the syntax.

```codelaw
match:
  all:
    - kind: function_declaration
    - regex: '^async function'
    - has:
        field: parameters
        regex: '^\(\s*[^)\s]'
    - inside:
        kind: export_statement
    - inside:
        kind: program
        stopBy: end
        has:
          kind: expression_statement
          regex: '^["'']use server["'']'
    - not:
        has:
          field: body
          regex: '(?i)[A-Za-z_$]*schema\.(safeParse|parse|parseAsync)\(|\.safeParse\(|\.parseAsync\(|\bz\.|valibot|\bv\.(parse|safeParse)|validate|class-validator|plainToInstance|\.assert\(|ajv|typia|\byup\b|\bjoi\b'
message: "This server action uses its arguments without validating them"
```

## How to fix

1. Parse the arguments with a schema (`zod`, `valibot`) at the top of the action and use the parsed value only.
2. Check the caller next: read the session and refuse when the user may not do this (`const session = await auth(); if (!session) throw …`).
3. Return a small, typed result; never return a database row whole.

## Examples

### TypeScript

#### Don't

```ts
"use server";
export async function createPost(input: FormData) {
  await db.insert(posts).values({ title: String(input.get("title")) });
}
```

#### Do

```ts
"use server";
export async function createPost(input: FormData) {
  const data = postSchema.parse({ title: input.get("title") });
  await db.insert(posts).values(data);
}
```

### TSX

#### Don't

```tsx
"use server";
export async function createPost(input: FormData) {
  await db.insert(posts).values({ title: String(input.get("title")) });
}
```

#### Do

```tsx
"use server";
export async function createPost(input: FormData) {
  const data = postSchema.parse({ title: input.get("title") });
  await db.insert(posts).values(data);
}
```
