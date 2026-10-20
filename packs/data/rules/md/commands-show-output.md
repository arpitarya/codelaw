---
id: md/commands-show-output
title: "No $ before shell commands that show no output"
languages: [markdown]
fix: auto
stability: preview
ported_from: [markdownlint/MD014]
since: 0.1.0
---

A code block in which every line is a command starting with `$` drops the `$`. A block that also shows output keeps it, since the `$` then tells commands from output.

**Why:** Readers copy commands straight from the block, and a pasted `$` makes every one of them fail.

```codelaw
native: md/commands-show-output
```

## Examples

### Markdown

#### Don't

````md
```sh
$ npm install
$ npm test
```
````

#### Do

````md
```sh
npm install
npm test
```
````
