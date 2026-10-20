# yaml/no-ambiguous-numbers — edge cases

### case: plain numbers and times that are not base 60

```yaml
a: 0
b: 10:75
c: 0:30
d: 1.5
```

```expect
(none)
```

### case: negative leading zero

```yaml
offset: -010
```

```expect
1:9 yaml/no-ambiguous-numbers
```
