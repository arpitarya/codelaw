---
id: md/fence-language
title: "Fenced code blocks name their language"
languages: [markdown]
fix: manual
fix_reason: "which language the code is in can't always be told from the code"
stability: preview
ported_from: [markdownlint/MD040]
since: 0.1.0
---

A fenced code block names its language after the opening fence: ` ```ts `, ` ```sh `, or ` ```text ` for plain output.

**Why:** The language drives syntax highlighting, and lets tools — codelaw's `lint_code_blocks` among them — check the code inside.

```codelaw
match:
  kind: fenced_code_block
  not:
    has: { kind: info_string }
message: "Name the code's language after the opening fence, or `text` for plain output"
```

## How to fix

1. Work out what the block holds.
2. Add its language after the opening fence, or `text` when it is plain output.

## Examples

### Markdown

#### Don't

````md
```
npm install codelaw
```
````

#### Do

````md
```sh
npm install codelaw
```
````
