---
id: css/string-no-newline
title: "No line breaks inside selector strings"
languages: [css]
fix: manual
fix_reason: "whether the line break was meant is the author's call"
stability: preview
options:
  ignore: { type: "string[]", default: ["at-rule-preludes", "declaration-values"], description: "contexts not to check: `at-rule-preludes`, `declaration-values`" }
ported_from: [stylelint/string-no-newline]
since: 0.1.0
---

A string in a selector does not contain an unescaped line break. With the default options, strings in at-rule preludes and declaration values are not checked.

**Why:** A raw line break ends a CSS string early, so the selector is invalid and the rule is dropped.

```codelaw
native: css/string-no-newline
```

## How to fix

1. Join the string onto one line, or escape the line break as `\A`.

## Examples

### CSS

#### Don't

```css
[title="one
two"] {
  color: red;
}
```

#### Do

```css
[title="one two"] {
  color: red;
}
```
