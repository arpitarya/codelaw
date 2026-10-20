---
id: md/no-bare-urls
title: "URLs are links, not bare text"
languages: [markdown]
fix: auto
stability: preview
ported_from: [markdownlint/MD034]
since: 0.1.0
---

A URL in running text is wrapped in angle brackets, `<https://…>`, or written as a link. URLs in code spans, links, autolinks and HTML are left alone.

**Why:** CommonMark only links a URL in angle brackets; a bare one is plain text in many renderers.

```codelaw
native: md/no-bare-urls
```

## Examples

### Markdown

#### Don't

````md
The docs are at https://example.com/docs.
````

#### Do

````md
The docs are at <https://example.com/docs>.
````

## Differences from the source

Bare e-mail addresses are not reported.
