---
id: css/syntax-string-no-invalid
title: "`@property` syntax strings are valid"
languages: [css, scss]
fix: manual
fix_reason: "the intended type is the author's call"
stability: preview
ported_from: [stylelint/syntax-string-no-invalid]
since: 0.1.0
---

The `syntax` of a `@property` rule is `"*"` or a `|`-separated list of supported `<type>` names and keywords, each with an optional `+` or `#`.

**Why:** A browser ignores a `@property` rule whose syntax is invalid, so the custom property is not registered.

```codelaw
native: css/syntax-string-no-invalid
```

## How to fix

1. Use a supported type name such as `<length>`, `<color>` or `<number>`.

## Examples

### CSS

#### Don't

```css
@property --size {
  syntax: "<size>";
  inherits: false;
  initial-value: 0px;
}
```

#### Do

```css
@property --size {
  syntax: "<length>";
  inherits: false;
  initial-value: 0px;
}
```
### SCSS

#### Don't

```scss
@property --size {
  syntax: "<size>";
  inherits: false;
  initial-value: 0px;
}
```

#### Do

```scss
@property --size {
  syntax: "<length>";
  inherits: false;
  initial-value: 0px;
}
```
