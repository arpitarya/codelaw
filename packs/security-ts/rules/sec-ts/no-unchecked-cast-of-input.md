---
id: sec-ts/no-unchecked-cast-of-input
title: "No type assertion on input that nothing has checked"
languages: [typescript, tsx]
fix: manual
fix_reason: "the schema that describes the input is written by the author"
stability: preview
security: { cwe: [20], owasp: ["A08:2025"], severity: medium }
since: 0.3.0
---

`JSON.parse(x) as T`, `req.body as T` and `(await res.json()) as T` are not used where the function has no runtime validator (zod, valibot, class-validator, ajv).

**Why:** A type assertion is a promise to the compiler, not a check: the data can be anything the client sent, and the rest of the code trusts the type (CWE-20). This reads one function's syntax; without types it cannot see a validator that runs elsewhere.

```codelaw
match:
  all:
    - kind: as_expression
    - regex: '^(JSON\.parse\(|\(?await\s+[\w$.]+\.json\(\)|(req|request)\.(body|query|params)\b|ctx\.request\.body\b)'
    - not:
        inside:
          stopBy: end
          any:
            - kind: function_declaration
            - kind: arrow_function
            - kind: method_definition
            - kind: function_expression
          has:
            stopBy: end
            regex: '(?i)[A-Za-z_$]*schema\.(safeParse|parse|parseAsync)\(|\.safeParse\(|\.parseAsync\(|\bz\.|valibot|\bv\.(parse|safeParse)|validate|class-validator|plainToInstance|\.assert\(|ajv|typia|\byup\b|\bjoi\b'
message: "Input is asserted to a type here without a check; the type is a promise nobody verified"
```

## How to fix

1. Parse the value with a schema (`zod`, `valibot`, `ajv`) and use the parsed result, whose type is inferred from the schema.
2. In NestJS, take a DTO class with `class-validator` decorators and a `ValidationPipe`, not a cast.
3. For `JSON.parse`, check the shape of the result before it is used as `T`.

## Examples

### TypeScript

#### Don't

```ts
function create(req: Request) {
  const dto = req.body as CreateUserDto;
  return save(dto);
}
```

#### Do

```ts
function create(req: Request) {
  const dto = createUserSchema.parse(req.body);
  return save(dto);
}
```

### TSX

#### Don't

```tsx
function create(req: Request) {
  const dto = req.body as CreateUserDto;
  return save(dto);
}
```

#### Do

```tsx
function create(req: Request) {
  const dto = createUserSchema.parse(req.body);
  return save(dto);
}
```
