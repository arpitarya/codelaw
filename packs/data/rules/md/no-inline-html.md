---
id: md/no-inline-html
title: "No raw HTML in Markdown"
languages: [markdown]
fix: manual
fix_reason: "the Markdown equivalent, if there is one, needs the author"
stability: preview
options:
  allowed_elements: { type: "string[]", default: [], description: "element names that may be used, such as `details` or `kbd`" }
ported_from: [markdownlint/MD033]
since: 0.1.0
---

A Markdown document doesn't use raw HTML elements, except those listed in `allowed_elements`. HTML comments are not reported.

**Why:** Raw HTML ties the document to renderers that pass it through; many (GitHub among them) strip some of it, and it hides the structure Markdown tools read.

```codelaw
native: md/no-inline-html
```

## How to fix

1. Rewrite the element in Markdown where an equivalent exists.
2. Where none does, add the element to `allowed_elements`.

## Examples

### Markdown

#### Don't

````md
<b>Note:</b> back up first.
````

#### Do

````md
**Note:** back up first.
````

## Notes

This rule is in `data/conventions`, not `md/recommended`: most READMEs rely on HTML for layout (centred logos, `<details>`), so it is a house style rather than a correctness check.
