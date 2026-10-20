---
id: md/first-line-heading
title: "A document starts with its title heading"
languages: [markdown]
fix: manual
fix_reason: "the title text needs the author"
stability: preview
options:
  level: { type: "integer", default: 1, description: "the level of the title heading" }
  allow_preamble: { type: "boolean", default: false, description: "allow text before the title heading" }
ported_from: [markdownlint/MD041]
since: 0.1.0
---

The first block of a document, after any front matter and HTML comments, is a level-1 heading — Markdown, or an HTML `<h1>`. A front matter `title:` counts as the heading.

**Why:** The first heading names the document for readers, search results and tools that build navigation from it.

```codelaw
native: md/first-line-heading
```

## How to fix

1. Write a heading that names the document.
2. Put it first, above any introduction.

## Examples

### Markdown

#### Don't

````md
This tool formats code.

# Formatter
````

#### Do

````md
# Formatter

This tool formats code.
````
