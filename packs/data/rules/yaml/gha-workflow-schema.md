---
id: yaml/gha-workflow-schema
title: "GitHub Actions workflows match their schema"
languages: [yaml]
files: ["**/.github/workflows/*.yml", "**/.github/workflows/*.yaml"]
fix: manual
fix_reason: "a misshapen job or step has several possible meanings; the author picks the one they intended"
stability: preview
ported_from: [schemastore/github-workflow.json]
since: 0.1.0
---

A GitHub Actions workflow matches SchemaStore's schema for workflows: every key of a trigger, a job and a step is one GitHub reads, with a value of the form it takes.

**Why:** GitHub rejects a malformed workflow only when it is pushed, and a misspelt key such as `runs_on` or `timeout-minute` fails the whole workflow file, so the mistake shows up as a red run instead of in review.

```codelaw
target: $
assert: { schema: schemas/github-workflow }
message: "$ERROR"
```

## How to fix

1. Read the message: it names the key and what the schema expects, and suggests the key meant when the name is misspelt.
2. Correct the key or the value.
3. Run `codelaw check` on the file again until it is clean.

## Examples

### YAML

#### Don't

```yaml path=.github/workflows/ci.yml
on: push
jobs:
  test:
    runs_on: ubuntu-latest
    steps:
      - run: npm test
```

#### Do

```yaml path=.github/workflows/ci.yml
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: npm test
```

## Notes

The schema is SchemaStore's `github-workflow.json` schema, shipped with the pack in `data/schemas/github-workflow.json` with its descriptions removed; nothing is fetched. YAML is read as YAML 1.2, as GitHub reads it, so `on:` is a key. `oneOf` is read as `anyOf`.
