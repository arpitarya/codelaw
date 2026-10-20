---
id: yaml/gha-action-schema
title: "GitHub Actions action metadata matches its schema"
languages: [yaml]
files: ["**/action.yml", "**/action.yaml"]
fix: manual
fix_reason: "a misshapen input or run step has several possible meanings; the author picks the one they intended"
stability: preview
ported_from: [schemastore/github-action.json]
since: 0.1.0
---

An `action.yml` matches SchemaStore's schema for action metadata: `name`, `description` and `runs` are present, and the inputs, outputs and steps have the keys and forms GitHub reads.

**Why:** GitHub reads an action's metadata only when a workflow uses it, so a misspelt `steps:` or `using:` breaks every workflow that calls the action, not the action's own repository.

```codelaw
target: $
assert: { schema: schemas/github-action }
message: "$ERROR"
```

## How to fix

1. Read the message: it names the key and what the schema expects.
2. Add the missing key, or correct the key or the value.
3. Run `codelaw check` on the file again until it is clean.

## Examples

### YAML

#### Don't

```yaml path=action.yml
name: Setup
description: Set the toolchain up
runs:
  using: composite
  step:
    - run: ./setup.sh
      shell: bash
```

#### Do

```yaml path=action.yml
name: Setup
description: Set the toolchain up
runs:
  using: composite
  steps:
    - run: ./setup.sh
      shell: bash
```

## Notes

The schema is SchemaStore's `github-action.json` schema, shipped with the pack in `data/schemas/github-action.json` with its descriptions removed; nothing is fetched. `oneOf` is read as `anyOf`.
