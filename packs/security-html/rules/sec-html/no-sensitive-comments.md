---
id: sec-html/no-sensitive-comments
title: "No comment that exposes a secret or an internal host"
languages: [html]
fix: auto
stability: preview
security: { cwe: [615], owasp: ["A02:2025"], severity: low }
since: 0.2.0
---

An HTML comment does not hold a password, key, token, private IP address, internal host name, or a note to remove it before release.

**Why:** Comments are sent to every visitor, so a developer's note becomes a disclosure.

```codelaw
native: sec-html/no-sensitive-comments
```

## Examples

### HTML

#### Don't

```html
<!-- password: hunter2 -->
<p>Welcome</p>
```

#### Do

```html
<p>Welcome</p>
```
