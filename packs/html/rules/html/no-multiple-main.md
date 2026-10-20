---
id: html/no-multiple-main
title: "One visible <main> per document"
languages: [html]
fix: manual
fix_reason: "which one is the main content is a design decision"
stability: preview
ported_from: [html-validate/no-multiple-main]
since: 0.1.0
---

A document has at most one `<main>` without the `hidden` attribute.

**Why:** Assistive technology jumps to "the" main content; with two, users land on one and miss the other.

```codelaw
native: html/no-multiple-main
```

## How to fix

1. Keep one `<main>`; make the others `<section>`, `<aside>` or similar, or hide inactive ones with `hidden`.

## Examples

### HTML

#### Don't

```html
<main>Feed</main>
<main>Sidebar</main>
```

#### Do

```html
<main>Feed</main>
<aside>Sidebar</aside>
```
