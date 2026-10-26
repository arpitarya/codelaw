---
id: sec-ts/no-unchecked-cast-of-input
title: "No type assertion on input that nothing has checked"
languages: [typescript, tsx]
fix: manual
fix_reason: "the schema that describes the input is written by the author"
stability: preview
security: { cwe: [20], owasp: ["A08:2025"], severity: medium }
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

`JSON.parse(x) as T`, `req.body as T` and `(await res.json()) as T` are not used where the function has no runtime validator (zod, valibot, class-validator, ajv).

**Your validator:** the function passes when it calls one of the `validators` anywhere in its body, or in a function around it: the built-in list (zod and valibot imports and a `*Schema.parse`, any `.safeParse`, `.parseAsync`, `.validate` or `.assert`, yup, joi, class-validator, class-transformer's `plainToInstance`, superstruct, typia) plus your own, each matched by name and by the module it is imported from, so a local `function validate(x) { return x }` does not count. `validators_mode = "replace"` makes yours the only ones that count. Add a project function with `validators = [{ name = "checkInput", module = "@acme/validation" }]`. A validator that a helper calls for you is not seen: name the helper.

**Why:** A type assertion is a promise to the compiler, not a check: the data can be anything the client sent, and the rest of the code trusts the type (CWE-20). This reads one function's syntax; without types it cannot see a validator that runs elsewhere.

```codelaw
match:
  all:
    - kind: as_expression
    - regex: '^(JSON\.parse\(|\(?await\s+[\w$.]+\.json\(\)|(req|request)\.(body|query|params)\b|ctx\.request\.body\b)'
where:
  $$MATCH: { not: { function_calls: validators } }
message: "Input is asserted to a type here without a check (add your validator to `validators`); the type is a promise nobody verified"
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
