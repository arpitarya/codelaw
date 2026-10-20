---
id: css/comment-no-empty
title: "No empty comments"
languages: [css]
fix: auto
stability: preview
ported_from: [stylelint/comment-no-empty]
since: 0.1.0
---

Every comment has some text.

**Why:** An empty comment carries nothing and is usually left over from an edit.

```codelaw
native: css/comment-no-empty
```

## Examples

### CSS

#### Don't

```css
/**/
a {
  color: red;
}
```

#### Do

```css
a {
  color: red;
}
```
