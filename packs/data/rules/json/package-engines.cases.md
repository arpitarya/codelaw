# json/package-engines — edge cases

### case: engines without node

```json
{
  "engines": {
    "npm": ">=10"
  }
}
```

```expect
2:14 json/package-engines
```

```fixed
{
  "engines": {
    "node": ">=22",
    "npm": ">=10"
  }
}
```
