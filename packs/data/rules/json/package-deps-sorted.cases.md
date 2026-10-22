# json/package-deps-sorted — edge cases

### case: every list is checked, and each is fixed on its own

```json
{
  "dependencies": { "b": "1", "a": "1" },
  "devDependencies": {
    "vitest": "^3.0.0",
    "typescript": "^5.0.0"
  },
  "peerDependencies": { "react": "*" }
}
```

```expect
2:3 json/package-deps-sorted
3:3 json/package-deps-sorted
```

```fixed
{
  "dependencies": { "a": "1", "b": "1" },
  "devDependencies": {
    "typescript": "^5.0.0",
    "vitest": "^3.0.0"
  },
  "peerDependencies": { "react": "*" }
}
```

### case: a list of bundled names

```json
{
  "bundleDependencies": ["zod", "ajv"]
}
```

```expect
2:3 json/package-deps-sorted
```

```fixed
{
  "bundleDependencies": ["ajv", "zod"]
}
```

### case: a comment between entries keeps the fix from running

```json
{
  "dependencies": {
    "b": "1",
    // pinned until the next major
    "a": "1"
  }
}
```

```expect
2:3 json/package-deps-sorted
```

### case: the text around the list is untouched

```json
{
  // build tooling
  "name":    "x",
  "dependencies": {"z": "1",
                   "a": "2"},
  "version": "1.0.0"
}
```

```expect
4:3 json/package-deps-sorted
```

```fixed
{
  // build tooling
  "name":    "x",
  "dependencies": {"a": "2",
                   "z": "1"},
  "version": "1.0.0"
}
```
