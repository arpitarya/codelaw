---
id: md/no-reversed-links
title: "Links are written [text](url), not (text)[url]"
languages: [markdown]
fix: auto
stability: preview
ported_from: [markdownlint/MD011]
since: 0.1.0
---

Text written as `(text)[url]` is a link with its brackets swapped. Footnote references such as `(note)[^1]` are not reported.

**Why:** The reversed form doesn't render as a link: readers see the brackets and the raw URL.

```codelaw
native: md/no-reversed-links
```

## Examples

### Markdown

#### Don't

````md
Read (the guide)[https://example.com/guide].
````

#### Do

````md
Read [the guide](https://example.com/guide).
````
