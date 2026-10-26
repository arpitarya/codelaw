---
id: sec-ts/trpc-public-mutation
title: "A tRPC mutation is not public"
languages: [typescript, tsx]
fix: manual
fix_reason: "which procedure carries the auth middleware is named by the author"
stability: preview
security: { cwe: [862], owasp: ["A01:2025"], severity: high }
since: 0.3.0
---

`publicProcedure` is not used for a `.mutation(…)`; `protectedProcedure` (a procedure with an auth middleware) is.

**Why:** A mutation changes data; a public one lets anyone who finds the route change it (CWE-862).

```codelaw
match:
  all:
    - pattern: $C.mutation($F)
    - regex: '^publicProcedure\b'
message: "This mutation is public; anyone can call it"
```

## How to fix

1. Use `protectedProcedure` (one that runs the session check as middleware) for every mutation, and `.use(isAdmin)` where a role is needed.
2. If a mutation really is public (a contact form), leave `publicProcedure` and suppress this finding with a reason that says why, and rate-limit it.

## Examples

### TypeScript

#### Don't

```ts
export const create = publicProcedure.input(schema).mutation(({ input }) => save(input));
```

#### Do

```ts
export const create = protectedProcedure.input(schema).mutation(({ input }) => save(input));
```

### TSX

#### Don't

```tsx
export const create = publicProcedure.input(schema).mutation(({ input }) => save(input));
```

#### Do

```tsx
export const create = protectedProcedure.input(schema).mutation(({ input }) => save(input));
```
