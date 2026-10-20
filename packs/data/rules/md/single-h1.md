---
id: md/single-h1
title: "One level-1 heading per document"
languages: [markdown]
fix: manual
fix_reason: "which heading is the title, and what level the others become, needs the author"
stability: preview
options:
  level: { type: "integer", default: 1, description: "the level of the title heading" }
ported_from: [markdownlint/MD025]
since: 0.1.0
---

A document has one top-level heading, its title. Every later level-1 heading is reported; when the front matter has a `title:`, every level-1 heading is.

**Why:** The level-1 heading names the document; a second one splits the outline in two and gives the page two titles.

```codelaw
native: md/single-h1
```

## How to fix

1. Keep the heading that names the document at level 1.
2. Make the others level 2 and shift their subsections down to match.

## Examples

### Markdown

#### Don't

````md
# Guide

Intro.

# Usage

Steps.
````

#### Do

````md
# Guide

Intro.

## Usage

Steps.
````

## Differences from the source

markdownlint applies the rule only when the first line of the file is the level-1 heading; here the first level-1 heading anywhere is the title, so a README that starts with badges is checked too.
