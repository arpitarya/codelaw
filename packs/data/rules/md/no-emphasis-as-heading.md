---
id: md/no-emphasis-as-heading
title: "Headings are headings, not bold lines"
languages: [markdown]
fix: manual
fix_reason: "which heading level the line should become depends on the outline"
stability: preview
options:
  punctuation: { type: "string", default: ".,;:!?。，；：！？", description: "a line ending in one of these is a sentence, not a heading" }
ported_from: [markdownlint/MD036]
since: 0.1.0
---

A paragraph that is a single line of bold or italic text, standing alone, is a heading. Lines ending in punctuation are sentences and are not reported.

**Why:** Emphasis only looks like a heading: it doesn't appear in the outline, gets no anchor and isn't announced as a heading by screen readers.

```codelaw
native: md/no-emphasis-as-heading
```

## How to fix

1. Pick the level the section belongs at.
2. Replace the emphasis with a heading of that level.

## Examples

### Markdown

#### Don't

````md
# Guide

**Installation**

Run the installer.
````

#### Do

````md
# Guide

## Installation

Run the installer.
````
