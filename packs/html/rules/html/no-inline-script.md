---
id: html/no-inline-script
title: "No inline event handlers or javascript: URLs"
languages: [html]
fix: manual
fix_reason: "the handler has to move into a script file"
stability: preview
ported_from: [htmlhint/inline-script-disabled]
since: 0.1.0
---

Elements don't have inline event handlers (`onclick`, `onload`, …) or `javascript:` URLs.

**Why:** Inline script mixes behaviour into markup, can't be cached or tested, and is blocked by a strict Content Security Policy.

```codelaw
native: html/no-inline-script
```

## How to fix

1. Give the element an id or class.
2. Attach the listener with `addEventListener` in a script file.
3. Replace a `javascript:` link with a `<button>`.

## Examples

### HTML

#### Don't

```html
<button type="button" onclick="save()">Save</button>
<a href="javascript:void(0)">Menu</a>
```

#### Do

```html
<button type="button" id="save">Save</button>
<button type="button" id="menu">Menu</button>
```
