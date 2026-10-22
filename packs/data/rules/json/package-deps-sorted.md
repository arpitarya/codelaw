---
id: json/package-deps-sorted
title: "package.json dependency lists are sorted by name"
languages: [json]
files: ["**/package.json"]
fix: auto
stability: preview
since: 0.1.0
---

The `dependencies`, `devDependencies`, `peerDependencies`, `optionalDependencies` and `bundleDependencies` of a `package.json` are in name order.

**Why:** npm writes these lists sorted, so a hand-added entry out of order turns into an unrelated diff the next time npm touches the file; a sorted list is also quicker to scan.

```codelaw
target: [dependencies, devDependencies, peerDependencies, optionalDependencies, bundleDependencies, bundledDependencies]
assert: { sorted: true }
message: "Sort `$PATH` by package name"
fix: [sort-keys]
```

## Examples

### JSON

#### Don't

```json path=package.json
{
  "name": "app",
  "dependencies": {
    "react": "^19.0.0",
    "@tanstack/query": "^5.0.0",
    "lodash": "^4.17.21"
  }
}
```

#### Do

```json path=package.json
{
  "name": "app",
  "dependencies": {
    "@tanstack/query": "^5.0.0",
    "lodash": "^4.17.21",
    "react": "^19.0.0"
  }
}
```

## Notes

Names compare by Unicode code point, so scoped packages (`@scope/…`) come first. `bundleDependencies` may be a list of names; its items are sorted the same way. A list with comments between its entries is reported without a fix: the comments would be left behind while the entries move.
