---
id: json/tsconfig-schema
title: "tsconfig.json matches its schema"
languages: [json]
files: ["**/tsconfig.json", "**/tsconfig.*.json"]
fix: manual
fix_reason: "a misspelt or misshapen option has several possible meanings; the author picks the one they intended"
stability: preview
ported_from: [schemastore/tsconfig.json]
since: 0.1.0
---

A `tsconfig.json` matches SchemaStore's schema for it: every compiler option is one TypeScript knows, with a value of the type it takes.

**Why:** `tsc` reports an unknown option only when it runs, and many editors and bundlers that read `tsconfig.json` skip it silently, so a misspelt `"stirct": true` leaves the project less strict than it looks.

```codelaw
target: $
assert: { schema: schemas/tsconfig }
message: "$ERROR"
```

## How to fix

1. Read the message: it names the option and what the schema expects, and suggests the option meant when the name is misspelt.
2. Correct the name or the value; remove an option TypeScript no longer has.
3. Run `codelaw check` on the file again until it is clean.

## Examples

### JSON

#### Don't

```json path=tsconfig.json
{
  "compilerOptions": {
    "target": "es2022",
    "stirct": true,
    "noEmit": "true"
  }
}
```

#### Do

```json path=tsconfig.json
{
  "compilerOptions": {
    "target": "es2022",
    "strict": true,
    "noEmit": true
  }
}
```

## Notes

The schema is SchemaStore's `tsconfig.json` schema, shipped with the pack in `data/schemas/tsconfig.json` with its descriptions removed; nothing is fetched. Comments and trailing commas are allowed, as `tsc` allows them. `oneOf` is read as `anyOf`.
