---
id: json/package-scripts-sorted
title: "package.json scripts are sorted by name"
languages: [json]
files: ["**/package.json"]
fix: auto
stability: preview
since: 0.1.0
---

The `scripts` of a `package.json` are in name order. A `pre<name>` or `post<name>` script sorts next to the `<name>` script it hooks into.

**Why:** A sorted list is quicker to scan and gives smaller diffs when scripts are added; npm runs scripts by name, so their order means nothing to it.

```codelaw
native: json/package-scripts-sorted
```

## Examples

### JSON

#### Don't

```json path=package.json
{
  "name": "app",
  "scripts": {
    "test": "vitest",
    "build": "tsc",
    "pretest": "npm run build",
    "lint": "codelaw check"
  }
}
```

#### Do

```json path=package.json
{
  "name": "app",
  "scripts": {
    "build": "tsc",
    "lint": "codelaw check",
    "pretest": "npm run build",
    "test": "vitest"
  }
}
```

## Notes

Names compare by Unicode code point. A `scripts` object that uses `npm-run-all`, `run-s` or `run-p` is left alone: those runners expand patterns such as `build:*` in the order the scripts are written, so sorting would change what runs. A `scripts` object with comments in it is reported without a fix, because the comments would stay behind while the scripts move.
