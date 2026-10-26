---
id: sec-html/no-template-html-sink
title: "No template binding that writes HTML"
languages: [html]
fix: manual
fix_reason: "whether the value is trusted, and how to sanitise it, is the author's call"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

A template does not bind a value as HTML: Angular `[innerHTML]` and `[outerHTML]`, Vue `v-html`, Alpine `x-html`.

**Why:** The value is written into the page as markup, so any part of it a user controls is cross-site scripting.

```codelaw
native: sec-html/no-template-html-sink
```

## How to fix

1. Bind text instead (`{{ value }}`, `v-text`, `x-text`, `[textContent]`).
2. If it must be HTML, sanitise it with a maintained library such as DOMPurify and bind the sanitised result.

## Examples

### HTML

#### Don't

```html
<div [innerHTML]="comment"></div>
```

#### Do

```html
<div>{{ comment }}</div>
```
