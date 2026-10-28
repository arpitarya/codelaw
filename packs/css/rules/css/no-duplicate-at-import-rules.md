---
id: css/no-duplicate-at-import-rules
title: "No stylesheet imported twice"
languages: [css]
fix: suggest
fix_reason: "removing the later import moves its rules earlier in the cascade"
stability: stable
ported_from: [stylelint/no-duplicate-at-import-rules]
since: 0.1.0
---

Each stylesheet is imported once with the same media conditions.

**Why:** The second import loads the same rules again, which costs a request and can reorder the cascade.

```codelaw
native: css/no-duplicate-at-import-rules
```

## Examples

### CSS

#### Don't

```css
@import url("base.css");
@import url("base.css");
```

#### Do

```css
@import url("base.css");
```
