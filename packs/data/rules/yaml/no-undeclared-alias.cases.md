# yaml/no-undeclared-alias — edge cases

### case: anchor in an earlier document

```yaml
base: &base 1
---
copy: *base
```

```expect
3:7 yaml/no-undeclared-alias
```
