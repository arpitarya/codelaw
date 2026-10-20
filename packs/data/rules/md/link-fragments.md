---
id: md/link-fragments
title: "In-page links point at a heading or anchor that exists"
languages: [markdown]
fix: manual
fix_reason: "which section the link meant needs the author"
stability: preview
options:
  ignore_case: { type: "boolean", default: false, description: "compare fragments with anchors ignoring case" }
ported_from: [markdownlint/MD051]
since: 0.1.0
---

A link to `#fragment` in the same document matches an anchor: a heading's GitHub-style id (lower case, punctuation dropped, spaces as dashes, `-1`, `-2` … for repeats), a `{#id}` after a heading, or an HTML `id` or `name`. `#top` and GitHub line anchors such as `#L20` are always valid.

**Why:** A fragment that matches nothing scrolls nowhere; it is usually a heading that was renamed after the link was written.

```codelaw
native: md/link-fragments
```

## How to fix

1. Find the section the link meant.
2. Change the fragment to that heading's id, or add an anchor where the link should land.

## Examples

### Markdown

#### Don't

````md
# Guide

## Getting started

See [the first steps](#getting-startd).
````

#### Do

````md
# Guide

## Getting started

See [the first steps](#getting-started).
````
