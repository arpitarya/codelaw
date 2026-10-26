---
id: sec-ts/no-permissive-schema
title: "No request schema that lets anything through"
languages: [typescript, tsx]
fix: suggest
fix_reason: "a client that sends extra keys starts getting an error instead of having them kept; `z.any()` and class-validator's `validate` have no mechanical fix"
stability: preview
security: { cwe: [915, 20], owasp: ["A08:2025"], severity: high }
since: 0.3.0
---

A request schema does not use `z.any()` or `.passthrough()`, and class-validator's `validate` and `validateSync` are given `forbidUnknownValues`.

**Why:** A schema that accepts any value, or keeps unknown keys, lets a client send fields the code never meant to read, which then reach a model or a query (CWE-915, CWE-20).

```codelaw
match:
  any:
    - pattern: $S.passthrough()
    - pattern: z.any()
    - pattern: v.any()
    - all:
        - pattern: $V.validate($O)
        - inside:
            kind: program
            stopBy: end
            has:
              stopBy: end
              regex: 'class-validator'
    - all:
        - pattern: validate($O2)
        - inside:
            kind: program
            stopBy: end
            has:
              stopBy: end
              regex: 'class-validator'
    - all:
        - pattern: validateSync($O3)
        - inside:
            kind: program
            stopBy: end
            has:
              stopBy: end
              regex: 'class-validator'
    - all:
        - pattern: validate($O4, $OPT)
        - not:
            regex: 'forbidUnknownValues'
        - inside:
            kind: program
            stopBy: end
            has:
              stopBy: end
              regex: 'class-validator'
message: "This schema lets values or keys through that the code never named"
fix:
  - replace: "$S.strict()"
```

## How to fix


## Examples

### TypeScript

#### Don't

```ts
const User = z.object({ name: z.string() }).passthrough();
```

#### Do

```ts
const User = z.object({ name: z.string() }).strict();
```

#### Fixed

```ts
const User = z.object({ name: z.string() }).strict();
```

### TSX

#### Don't

```tsx
const User = z.object({ name: z.string() }).passthrough();
```

#### Do

```tsx
const User = z.object({ name: z.string() }).strict();
```

#### Fixed

```tsx
const User = z.object({ name: z.string() }).strict();
```
