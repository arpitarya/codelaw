---
id: html/close-order
title: "Elements are closed, and in the order they were opened"
languages: [html]
fix: manual
fix_reason: "where the missing end tag belongs depends on the intended structure"
stability: preview
ported_from: [html-validate/close-order, htmlhint/tag-pair]
since: 0.1.0
---

Every element whose end tag is required is closed, and no end tag closes an element that isn't open.

**Why:** Browsers repair mismatched tags by guessing, and the guess often nests content differently from what the author meant.

```codelaw
native: html/close-order
```

## How to fix

1. Find where the element named in the message should end.
2. Add its end tag there, or remove an end tag that closes nothing.

## Examples

### HTML

#### Don't

```html
<div>
  <span>Text
</div>
```

#### Do

```html
<div>
  <span>Text</span>
</div>
```

## Differences from the source

Elements whose end tag HTML lets you leave out (`<li>`, `<p>`, `<td>`, …) are not reported here; `html/no-implicit-close` reports them.
