---
id: md/relative-links
title: "Relative links point at a file that exists"
languages: [markdown]
fix: manual
fix_reason: "whether the file moved, was renamed or was never written needs the author"
stability: preview
since: 0.1.0
---

A link or image whose destination is a path, not a URL, names a file or folder of the project. The path is read from the linking file's folder (from the project root when it starts with `/`), without its `#fragment` or `?query`. Links with a scheme (`https:`, `mailto:`), in-page `#fragments`, template syntax and paths that leave the project are not checked.

**Why:** A relative link breaks silently when the file it names is moved or renamed; the reader gets a 404 where the documentation should be.

```codelaw
native: md/relative-links
```

## How to fix

1. Find where the file went: `git log --follow --stat -- <path>` shows a rename.
2. Point the link at the file's new path, or remove the link if the file is gone.

## Examples

### Markdown

#### Don't

````md
<!-- docs/guide.md -->
Start with [the setup](./setup.md), then read the [FAQ](../FAQ.md).
<!-- docs/setup.md -->
# Setup
````

#### Do

````md
<!-- docs/guide.md -->
Start with [the setup](./setup.md), then read the [FAQ](../FAQ.md).
<!-- docs/setup.md -->
# Setup
<!-- FAQ.md -->
# FAQ
````
