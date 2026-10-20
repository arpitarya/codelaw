---
id: html/no-redundant-role
title: "No role the element already has"
languages: [html]
fix: auto
stability: preview
ported_from: [html-validate/no-redundant-role]
tags: [a11y]
since: 0.1.0
---

An element's `role` is not the role it has anyway, such as `<ul role="list">`, `<button role="button">` or `<a href role="link">`.

**Why:** The attribute repeats the element and suggests the element was chosen for its looks.

```codelaw
native: html/no-redundant-role
```

## Examples

### HTML

#### Don't

```html
<ul role="list">
  <li>One</li>
</ul>
```

#### Do

```html
<ul>
  <li>One</li>
</ul>
```

## Differences from the source

Some Safari versions drop the list role of a `<ul>` styled with `list-style: none`; `role="list"` is a known workaround. Turn the rule off for such lists.
