---
id: css/selector-type-no-unknown
title: "No unknown type selectors"
languages: [css, scss]
fix: manual
fix_reason: "the intended element is the author's call"
stability: stable
options:
  ignore: { type: "string[]", default: ["custom-elements"], description: "`custom-elements` allows hyphenated names; `default-namespace` skips un-namespaced types" }
  ignoreTypes: { type: "string[]", default: [], description: "type selectors to accept; `/regex/` for a pattern" }
ported_from: [stylelint/selector-type-no-unknown]
since: 0.1.0
---

Every type selector is an HTML, SVG or MathML element. Custom elements (a hyphenated lowercase name) are allowed by default, and namespaced selectors are skipped.

**Why:** A misspelt element name matches nothing.

```codelaw
native: css/selector-type-no-unknown
```

## How to fix

1. Correct the element name.

## Examples

### CSS

#### Don't

```css
tabel {
  border-collapse: collapse;
}
```

#### Do

```css
table {
  border-collapse: collapse;
}
```
### SCSS

#### Don't

```scss
tabel {
  border-collapse: collapse;
}
```

#### Do

```scss
table {
  border-collapse: collapse;
}
```
