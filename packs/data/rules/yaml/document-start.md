---
id: yaml/document-start
title: "A YAML document starts with ---"
languages: [yaml]
exclude: ["**/*.md", "**/*.mdx"]
fix: auto
stability: preview
since: 0.1.0
---

Every YAML document begins with an explicit `---` line. Comments and directives may come before it. Markdown front matter is not checked: its `---` belongs to the Markdown file.

**Why:** An explicit start marks where a document begins, so a file can grow to hold several documents, and tools that concatenate YAML files keep the boundaries.

```codelaw
native: yaml/document-start
```

## Examples

### YAML

#### Don't

```yaml
name: build
on: push
```

#### Do

```yaml
---
name: build
on: push
```
