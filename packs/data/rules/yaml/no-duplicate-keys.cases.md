# yaml/no-duplicate-keys — edge cases

### case: quoted and plain spellings

```yaml
a: 1
"a": 2
```

```expect
2:1 yaml/no-duplicate-keys
```

### case: same key in sibling mappings

```yaml
dev:
  port: 1
prod:
  port: 2
```

```expect
(none)
```

### case: flow mapping

```yaml
point: {x: 1, y: 2, x: 3}
```

```expect
1:21 yaml/no-duplicate-keys
```
