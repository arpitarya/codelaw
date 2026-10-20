---
id: html/heading-level
title: "Headings start at <h1> and go down one level at a time"
languages: [html]
fix: manual
fix_reason: "changing a heading level changes its style and the outline"
stability: preview
options:
  allowMultipleH1: { type: "boolean", default: false, description: "allow more than one `<h1>`" }
  minInitialRank: { type: "string", default: "h1", description: "the lowest level the first heading may have (`h1`–`h6`, or `any`)" }
  sectioningRoots: { type: "string[]", default: ["dialog", "[role=\"dialog\"]", "[role=\"alertdialog\"]"], description: "selectors of elements that start their own outline" }
ported_from: [html-validate/heading-level]
since: 0.1.0
---

The first heading is `<h1>`, there is one `<h1>`, and each heading is at most one level below the one before it. A `<dialog>` starts its own outline.

**Why:** Screen-reader users navigate the heading outline; a skipped level looks like missing content.

```codelaw
native: html/heading-level
```

## How to fix

1. Change the heading to the level the message expects, and restyle it with CSS if it looked right before.

## Examples

### HTML

#### Don't

```html
<h1>Guide</h1>
<h3>Install</h3>
```

#### Do

```html
<h1>Guide</h1>
<h2>Install</h2>
```
