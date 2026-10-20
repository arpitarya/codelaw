---
id: md/no-alt-text
title: "Images have alt text"
languages: [markdown]
fix: manual
fix_reason: "describing the image needs a person who can see it"
stability: preview
ported_from: [markdownlint/MD045]
since: 0.1.0
---

A Markdown image has alt text: `![what it shows](…)`. HTML `<img>` elements are checked by `html/img-alt` in the HTML pack.

**Why:** Screen readers read the alt text in place of the image, and it shows when the image fails to load.

```codelaw
match:
  kind: image
  not:
    has: { kind: image_description }
message: "Describe the image in its alt text: `![what it shows](…)`"
```

## How to fix

1. Look at the image and say in a few words what it shows.
2. Put that between the brackets; for a purely decorative image, use HTML with `alt=""`.

## Examples

### Markdown

#### Don't

````md
![](diagram.png)
````

#### Do

````md
![The request flow from client to cache](diagram.png)
````

## Differences from the source

markdownlint also checks HTML `<img>`; here that is the HTML pack's `html/img-alt`, which lints HTML embedded in Markdown.
