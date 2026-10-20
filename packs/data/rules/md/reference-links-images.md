---
id: md/reference-links-images
title: "Reference links use a label that is defined"
languages: [markdown]
fix: manual
fix_reason: "where the reference should point needs the author"
stability: preview
options:
  ignored_labels: { type: "string[]", default: ["x"], description: "labels never reported (`x` is a task-list checkbox)" }
  shortcut_syntax: { type: "boolean", default: false, description: "also check `[label]` shortcuts, which plain bracketed text looks like" }
ported_from: [markdownlint/MD052]
since: 0.1.0
---

A reference link or image — `[text][label]`, `[label][]`, `![alt][label]` — uses a label that a `[label]: url` definition in the document defines. Labels match ignoring case and spacing.

**Why:** A reference to an undefined label isn't a link at all: it renders as the literal brackets.

```codelaw
native: md/reference-links-images
```

## How to fix

1. Check the label for a typo against the definitions.
2. Fix the label, or add the missing `[label]: url` definition.

## Examples

### Markdown

#### Don't

````md
Read [the spec][spec].
````

#### Do

````md
Read [the spec][spec].

[spec]: https://spec.commonmark.org/
````
