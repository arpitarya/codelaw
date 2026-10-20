---
id: yaml/no-ambiguous-numbers
title: "No numbers that YAML 1.1 and 1.2 read differently"
languages: [yaml]
fix: manual
fix_reason: "whether the octal or base-60 meaning or the YAML 1.2 meaning was intended needs the author"
stability: preview
since: 0.1.0
---

A value is not an unquoted integer with a leading zero (`0755`) or an unquoted base-60 form (`22:22`, `1:30:00`).

**Why:** YAML 1.1 reads `0755` as octal (493) and `22:22` as the number 1342; YAML 1.2 reads them as 755 and the string "22:22". The same file means different things to different loaders — the classic case is port mappings in `docker-compose.yml`.

```codelaw
native: yaml/no-ambiguous-numbers
```

## How to fix

1. For an octal number, write it as `0o755`; for a decimal one, drop the leading zero.
2. For text such as a port mapping or a time, quote it: `"22:22"`.

## Examples

### YAML

#### Don't

```yaml
mode: 0755
ports:
  - 22:22
```

#### Do

```yaml
mode: 0o755
ports:
  - "22:22"
```

## Differences from the source

The idea comes from the YAML 1.1 and 1.2 specifications.
