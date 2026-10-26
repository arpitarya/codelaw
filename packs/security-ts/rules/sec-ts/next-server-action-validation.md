---
id: sec-ts/next-server-action-validation
title: "A Next.js server action validates its input"
languages: [typescript, tsx]
fix: manual
fix_reason: "the schema and the authorisation check are written by the author"
stability: preview
security: { cwe: [20, 862], owasp: ["A01:2025"], severity: high }
since: 0.3.0
options:
  validators:
    type: "call[]"
    description: "calls that check input, any one of which in the function counts: a callee path or { name, module }; `validators_mode: replace` makes yours the only ones that count"
    default:
      - { name: "z.*", module: zod }
      - { name: "z.*.*", module: zod }
      - "*Schema.parse"
      - "*schema.parse"
      - "*.parseAsync"
      - "*.safeParse"
      - "*.safeParseAsync"
      - { name: "v.*", module: valibot }
      - { name: parse, module: valibot }
      - { name: safeParse, module: valibot }
      - { name: parseAsync, module: valibot }
      - { name: "yup.*", module: yup }
      - { name: "Joi.*", module: joi }
      - { name: "joi.*", module: joi }
      - "*.validate"
      - "*.validateSync"
      - "*.validateAsync"
      - { name: validate, module: [class-validator, superstruct] }
      - { name: validateOrReject, module: class-validator }
      - { name: validateSync, module: class-validator }
      - { name: plainToInstance, module: class-transformer }
      - { name: assert, module: superstruct }
      - { name: create, module: superstruct }
      - "*.assert"
      - { name: "typia.*", module: typia }
---

An exported function in a `"use server"` file that takes arguments calls a validator (`schema.parse`, `safeParse`, `validate`) on them.

**Your validator:** the function passes when it calls one of the `validators` anywhere in its body, or in a function around it: the built-in list (zod and valibot imports and a `*Schema.parse`, any `.safeParse`, `.parseAsync`, `.validate` or `.assert`, yup, joi, class-validator, class-transformer's `plainToInstance`, superstruct, typia) plus your own, each matched by name and by the module it is imported from, so a local `function validate(x) { return x }` does not count. `validators_mode = "replace"` makes yours the only ones that count. Add a project function with `validators = [{ name = "checkInput", module = "@acme/validation" }]`. A validator that a helper calls for you is not seen: name the helper.

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
where:
  $$MATCH: { not: { function_calls: validators } }
message: "This server action uses its arguments without calling a validator (add yours to `validators`)"
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
