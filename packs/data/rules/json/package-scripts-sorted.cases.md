# json/package-scripts-sorted — edge cases

### case: order-sensitive runner

```json
{
  "scripts": {
    "build": "run-s build:*",
    "build:js": "tsc",
    "build:css": "sass"
  }
}
```

```expect
(none)
```

### case: hooks sort next to their script

```json
{
  "scripts": {
    "prepare": "husky",
    "postbuild": "x",
    "build": "tsc",
    "prebuild": "rm -rf dist"
  }
}
```

```expect
2:3 json/package-scripts-sorted
```

```fixed
{
  "scripts": {
    "prebuild": "rm -rf dist",
    "build": "tsc",
    "postbuild": "x",
    "prepare": "husky"
  }
}
```
