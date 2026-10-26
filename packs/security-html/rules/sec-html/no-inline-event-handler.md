---
id: sec-html/no-inline-event-handler
title: "No inline event handlers"
languages: [html]
fix: manual
fix_reason: "the handler's code has to move into a script that attaches it"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: medium }
since: 0.2.0
---

No element has an inline event handler (`onclick`, `onerror`, `onload`, …).

**Why:** An inline handler runs script from an attribute, so a strict Content Security Policy that blocks inline script blocks it too, and a value interpolated into it becomes cross-site scripting.

```codelaw
native: sec-html/no-inline-event-handler
```

## How to fix

1. Give the element an id or a class.
2. Attach the listener with `addEventListener` in a script file.
3. Delete the `on…` attribute.

## Examples

### HTML

#### Don't

```html
<button type="button" onclick="save()">Save</button>
```

#### Do

```html
<button type="button" id="save">Save</button>
```
