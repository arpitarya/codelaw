---
id: sec-html/no-unsafe-template-interpolation
title: "No template syntax that turns escaping off"
languages: [html]
fix: manual
fix_reason: "whether the value is trusted, and how to sanitise it, is the author's call"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

A template does not turn escaping off: Handlebars `{{{ }}}`, Jinja and Nunjucks `|safe`, Django `{% autoescape off %}`, EJS `<%-`.

**Why:** The value is written into the page as markup, so any part of it a user controls is cross-site scripting.

```codelaw
native: sec-html/no-unsafe-template-interpolation
```

## How to fix

1. Use the escaping form: `{{ value }}` (Handlebars, Jinja, Django), `<%= value %>` (EJS).
2. If the value must be markup, sanitise it with a maintained library first and mark only the sanitised result safe.

## Examples

### HTML

#### Don't

```html
<p>{{{ comment }}}</p>
```

#### Do

```html
<p>{{ comment }}</p>
```
