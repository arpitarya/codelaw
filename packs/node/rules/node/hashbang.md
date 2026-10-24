---
id: node/hashbang
title: "Commands start with a Node.js hashbang, and nothing else has one"
languages: [javascript, jsx]
fix: suggest
fix_reason: "removing the hashbang of a script someone runs directly (`./tool.js`) stops it running"
stability: preview
ported_from: [eslint-plugin-n/hashbang]
since: 0.1.0
---

A file the nearest `package.json` names in `bin` starts with `#!/usr/bin/env node`; a file it doesn't name has no hashbang. Without a `package.json` above the file, nothing is checked.

**Why:** npm links a `bin` entry as an executable, which the shell runs through its hashbang: without one the command fails, or runs under the wrong interpreter. A hashbang on a library file is noise that suggests it is a command.

```codelaw
native: node/hashbang
```

## How to fix

1. On a command `bin` names, make the first line `#!/usr/bin/env node`.
2. On any other file, remove the hashbang, or add the file to `bin` if it is a command.

## Examples

### JavaScript

#### Don't

```js
// package.json
{ "name": "tool", "bin": { "tool": "./bin/tool.js" } }
// bin/tool.js
console.log("hello");
```

#### Do

```js
// package.json
{ "name": "tool", "bin": { "tool": "./bin/tool.js" } }
// bin/tool.js
#!/usr/bin/env node
console.log("hello");
```

### JSX

#### Don't

```jsx
// package.json
{ "name": "tool", "bin": "./cli.jsx" }
// lib/view.jsx
#!/usr/bin/env node
export const View = () => <p>hello</p>;
```

#### Do

```jsx
// package.json
{ "name": "tool", "bin": "./cli.jsx" }
// lib/view.jsx
export const View = () => <p>hello</p>;
```

## Differences from the source

TypeScript sources are not checked: eslint-plugin-n maps them to their built JavaScript through `convertPath`, which this rule has no option for yet. The source's checks for a byte-order mark and CRLF line endings in a command are not ported, and there is no `additionalExecutables` or `executableMap` option.
