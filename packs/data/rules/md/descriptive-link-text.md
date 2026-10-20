---
id: md/descriptive-link-text
title: "Link text says where the link goes"
languages: [markdown]
fix: manual
fix_reason: "describing the target needs the author"
stability: preview
options:
  prohibited_texts: { type: "string[]", default: ["click here", "here", "link", "more"], description: "link texts that say nothing about the target, compared ignoring case and punctuation" }
ported_from: [markdownlint/MD059]
since: 0.1.0
---

A link's text is not a generic phrase such as "click here", "here", "link" or "more".

**Why:** Screen-reader users often jump from link to link, hearing only the link text; "here" tells them nothing.

```codelaw
native: md/descriptive-link-text
```

## How to fix

1. Say in a few words what the link leads to.
2. Make those words the link text.

## Examples

### Markdown

#### Don't

````md
For details, click [here](docs/setup.md).
````

#### Do

````md
For details, see [the setup guide](docs/setup.md).
````
