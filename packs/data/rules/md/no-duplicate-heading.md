---
id: md/no-duplicate-heading
title: "No two sibling headings with the same text"
languages: [markdown]
fix: manual
fix_reason: "which heading to rename, and to what, needs the author"
stability: preview
options:
  siblings_only: { type: "boolean", default: true, description: "only compare headings under the same parent; `false` compares every heading in the document" }
ported_from: [markdownlint/MD024]
since: 0.1.0
---

Headings under the same parent heading have different text.

**Why:** Readers and tables of contents tell sections apart by their headings, and repeated headings get anchors (`#setup`, `#setup-1`) that are easy to link to the wrong one.

```codelaw
native: md/no-duplicate-heading
```

## How to fix

1. Decide whether the two sections are one topic; if so, merge them.
2. Otherwise rename one heading to say how it differs.

## Examples

### Markdown

#### Don't

````md
# Tools

## Setup

Install the CLI.

## Setup

Configure it.
````

#### Do

````md
# Tools

## Setup

Install the CLI.

## Configuration

Configure it.
````

## Differences from the source

`siblings_only` defaults to `true` here (markdownlint defaults it to `false`), so a changelog's repeated `### Fixed` under each version is not reported.
