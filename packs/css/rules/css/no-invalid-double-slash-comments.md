---
id: css/no-invalid-double-slash-comments
title: "No `//` comments"
languages: [css]
fix: manual
fix_reason: "turning it into a comment changes what applies"
stability: stable
ported_from: [stylelint/no-invalid-double-slash-comments]
since: 0.1.0
---

The sheet has no `//` comments: CSS has only `/* … */` comments.

**Why:** CSS reads `//` as part of a selector or property, so the rule or declaration after it is dropped.

```codelaw
native: css/no-invalid-double-slash-comments
```

## How to fix

1. Wrap the comment in `/* … */`, or delete it.

## Examples

### CSS

#### Don't

```css
a {
  // color: red;
  top: 0;
}
```

#### Do

```css
a {
  /* color: red; */
  top: 0;
}
```
