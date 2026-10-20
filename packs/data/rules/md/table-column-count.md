---
id: md/table-column-count
title: "Table rows have as many cells as the header"
languages: [markdown]
fix: manual
fix_reason: "which cell is missing or extra needs the author"
stability: preview
ported_from: [markdownlint/MD056]
since: 0.1.0
---

Every row of a pipe table has the same number of cells as its header row.

**Why:** Renderers drop extra cells and leave missing ones empty, so a misplaced pipe silently loses or shifts data.

```codelaw
native: md/table-column-count
```

## How to fix

1. Compare the row with the header.
2. Add the missing cells or remove the extra ones, or add a header column if the data needs it.

## Examples

### Markdown

#### Don't

````md
| Name | Type |
| ---- | ---- |
| id | number | required |
````

#### Do

````md
| Name | Type | Note |
| ---- | ---- | ---- |
| id | number | required |
````
