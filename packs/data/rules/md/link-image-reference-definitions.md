---
id: md/link-image-reference-definitions
title: "Every reference definition is used, and defined once"
languages: [markdown]
fix: auto
stability: preview
options:
  ignored_definitions: { type: "string[]", default: ["//"], description: "labels never reported (`[//]: # (…)` is a comment idiom)" }
ported_from: [markdownlint/MD053]
since: 0.1.0
---

A `[label]: url` definition is referred to by some link or image, and no label is defined twice. The fix deletes the unused or repeated definition.

**Why:** A definition nothing uses is dead text that goes stale, and only the first of two definitions counts, so the second one misleads.

```codelaw
native: md/link-image-reference-definitions
```

## Examples

### Markdown

#### Don't

````md
Read [the spec][spec].

[spec]: https://spec.commonmark.org/
[old]: https://example.com/old
````

#### Do

````md
Read [the spec][spec].

[spec]: https://spec.commonmark.org/
````
