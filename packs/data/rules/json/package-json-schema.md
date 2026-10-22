---
id: json/package-json-schema
title: "package.json matches its schema"
languages: [json]
files: ["**/package.json"]
fix: manual
fix_reason: "a value the schema rejects is wrong in a way only its author knows how to correct"
stability: preview
ported_from: [schemastore/package.json]
since: 0.1.0
---

A `package.json` matches SchemaStore's schema for it: every field has the type and the form npm reads, and `name` and `version` are written the way the registry accepts.

**Why:** npm and the tools that read `package.json` ignore or misread a field of the wrong shape — a `bin` that is a list, a `files` that is a string, an `engines` range that is a number — so the mistake surfaces later, far from the line that caused it.

```codelaw
target: $
assert: { schema: schemas/package }
message: "$ERROR"
```

## How to fix

1. Read the message: it names the field and the form the schema expects.
2. Rewrite the value in that form, or remove the field if it was a mistake.
3. Run `codelaw check` on the file again until it is clean.

## Examples

### JSON

#### Don't

```json path=package.json
{
  "name": "app",
  "version": "1.0.0",
  "files": "dist",
  "private": "yes"
}
```

#### Do

```json path=package.json
{
  "name": "app",
  "version": "1.0.0",
  "files": ["dist"],
  "private": true
}
```

## Notes

The schema is SchemaStore's `package.json` schema, shipped with the pack in `data/schemas/package.json` with its descriptions removed; nothing is fetched. Fields whose schema lives in another SchemaStore file (`eslintConfig`, `prettier`, `stylelint`, `ava`, `release`, `nodemonConfig`, …) accept any value. `oneOf` is read as `anyOf`, and a pattern the engine's regular expressions can't read (look-around) accepts every value.
