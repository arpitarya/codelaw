---
id: sec-ts/no-any-request-handler
title: "No request typed any"
languages: [typescript, tsx]
fix: manual
fix_reason: "the real shape of the request is known only to the author"
stability: preview
security: { cwe: [20], owasp: ["A06:2025"], severity: medium }
since: 0.3.0
---

A route handler's request parameter or `@Body()`, `@Query()`, `@Param()` and `@Headers()` argument is not typed `any`.

**Why:** `any` turns the type checker off for everything that came from the client, so a wrong shape is not caught where it enters (CWE-20). This reads the written type only; without type information it cannot see an `any` that is inferred.

```codelaw
match:
  any:
    - kind: required_parameter
      regex: '^(req|request)\s*:\s*any$'
    - kind: required_parameter
      regex: '^@(Body|Query|Param|Headers)\([^)]*\)\s*[\w$]+\s*:\s*any$'
message: "A request is typed `any`; the compiler cannot check what the client sent"
```

## How to fix

1. Type the request: `Request<Params, ResBody, ReqBody, Query>` in Express, a DTO class in NestJS.
2. Validate the body at run time as well (see `sec-ts/no-unchecked-cast-of-input`): a type only describes what the code expects.

## Examples

### TypeScript

#### Don't

```ts
app.post("/users", (req: any, res: Response) => {
  save(req.body);
});
```

#### Do

```ts
app.post("/users", (req: Request<{}, {}, CreateUser>, res: Response) => {
  save(req.body);
});
```

### TSX

#### Don't

```tsx
app.post("/users", (req: any, res: Response) => {
  save(req.body);
});
```

#### Do

```tsx
app.post("/users", (req: Request<{}, {}, CreateUser>, res: Response) => {
  save(req.body);
});
```
