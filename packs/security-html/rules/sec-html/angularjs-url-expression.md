---
id: sec-html/angularjs-url-expression
title: "AngularJS URLs use ng-href and ng-src"
languages: [html]
fix: suggest
fix_reason: "the expression's value should be checked too; the rename is only the first step"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: medium }
since: 0.2.0
---

In a page that uses AngularJS, `href="{{…}}"` and `src="{{…}}"` are written `ng-href` and `ng-src`.

**Why:** The browser requests the literal `{{…}}` before AngularJS fills it in, and an unvetted expression in a URL is an injection point; the `ng-` forms are set only after interpolation.

```codelaw
native: sec-html/angularjs-url-expression
```

## Examples

### HTML

#### Don't

```html
<div ng-app><a href="{{ profileUrl }}">Profile</a></div>
```

#### Do

```html
<div ng-app><a ng-href="{{ profileUrl }}">Profile</a></div>
```
