# yaml/no-ambiguous-booleans — edge cases

### case: GitHub Actions trigger key

```yaml
on:
  push:
    branches: [main]
```

```expect
(none)
```

### case: quoted and sequence values

```yaml
a: "yes"
b: [on, 'off']
```

```expect
2:5 yaml/no-ambiguous-booleans
```

```fixed
a: "yes"
b: ["on", 'off']
```
