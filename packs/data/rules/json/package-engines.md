---
id: json/package-engines
title: "package.json declares engines.node"
languages: [json]
files: ["**/package.json"]
fix: suggest
fix_reason: "the supported Node.js range is a project decision; the fix inserts the `node` option, which may not be the right range"
stability: preview
options:
  node: { type: "string", default: ">=22", description: "the range the fix writes into `engines.node`" }
since: 0.1.0
---

A `package.json` says which Node.js versions the package runs on, in `engines.node`.

**Why:** Without it, npm and package managers can't warn someone installing on an unsupported Node.js, and the package fails later with an error that doesn't say why.

```codelaw
native: json/package-engines
```

## Examples

### JSON

#### Don't

```json path=package.json
{
  "name": "app",
  "version": "1.0.0"
}
```

#### Do

```json path=package.json
{
  "name": "app",
  "version": "1.0.0",
  "engines": {
    "node": ">=22"
  }
}
```
