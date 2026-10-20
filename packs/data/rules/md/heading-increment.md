---
id: md/heading-increment
title: "Heading levels go down one at a time"
languages: [markdown]
fix: suggest
fix_reason: "the fix raises the skipped heading and its subsections, which changes the outline readers and tools see"
stability: preview
ported_from: [markdownlint/MD001]
since: 0.1.0
---

A heading is at most one level deeper than the heading before it: `#` can be followed by `##`, not by `###`. When the front matter has a `title:`, it counts as the level-1 heading.

**Why:** Headings are the document's outline; a skipped level leaves a section with no parent, which confuses screen readers and tables of contents.

```codelaw
native: md/heading-increment
```

## Examples

### Markdown

#### Don't

````md
# Install

### From npm

Run the installer.
````

#### Do

````md
# Install

## From npm

Run the installer.
````

## Differences from the source

The fix renumbers the whole outline at once: a raised heading's subsections move up with it.
