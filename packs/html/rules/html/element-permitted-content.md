---
id: html/element-permitted-content
title: "Elements contain only content HTML allows there"
languages: [html]
fix: manual
fix_reason: "the right structure depends on what the markup is for"
stability: preview
ported_from: [html-validate/element-permitted-content]
since: 0.1.0
---

Each element contains only the elements its content model allows: no `<div>` inside a `<span>` or `<p>`, nothing interactive inside a link or button, no `<li>` directly inside a `<div>`.

**Why:** Browsers repair forbidden nesting by moving elements, so the DOM differs from the markup and styles and scripts miss.

```codelaw
native: html/element-permitted-content
```

## How to fix

1. Move the element named in the message out of the parent it can't be in.
2. Or change the parent to one that allows it (for example `<div>` instead of `<span>`).

## Examples

### HTML

#### Don't

```html
<p><div>Block</div></p>
<a href="/a"><button type="button">Go</button></a>
```

#### Do

```html
<div><p>Block</p></div>
<a href="/a">Go</a>
```

## Differences from the source

Obsolete elements (`<tt>`, `<font>`, `<acronym>`, …) have no content model in the element table, so html-validate reports them wherever they are; here they are left to `html/deprecated`, which reports each once.
