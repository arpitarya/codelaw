---
id: sec-html/form-csrf-token
title: "A POST form carries a CSRF token"
languages: [html]
fix: manual
fix_reason: "which token the server checks depends on its framework"
stability: preview
options:
  names: { type: "string[]", default: [], description: "extra names (field names or template tags) that count as a CSRF token, besides csrf, xsrf, authenticity_token, antiforgery, csrfmiddlewaretoken, __requestverificationtoken and _token" }
security: { cwe: [352], owasp: ["A01:2025"], severity: medium }
since: 0.2.0
---

A `<form method="post">` has a hidden anti-forgery field, or a template tag that writes one.

**Why:** Without a token another site can submit the form with the signed-in user's cookies.

```codelaw
native: sec-html/form-csrf-token
```

## How to fix

1. Add the framework's field: `{% csrf_token %}` (Django), `@csrf` (Laravel), `<%= csrf_meta_tags %>` / `authenticity_token` (Rails), `{{ form.hidden_tag() }}` (Flask-WTF), or a hidden `_csrf` input.
2. If the form posts to another site, ignore it with `exceptions`.

## Examples

### HTML

#### Don't

```html
<form method="post" action="/transfer"><input name="to"></form>
```

#### Do

```html
<form method="post" action="/transfer"><input type="hidden" name="csrf_token" value="{{ csrf_token }}"><input name="to"></form>
```
