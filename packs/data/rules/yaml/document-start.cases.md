# yaml/document-start — edge cases

### case: comment and directive before the marker

```yaml
# CI settings
%YAML 1.2
---
a: 1
```

```expect
(none)
```

### case: comment before content

```yaml
# settings
a: 1
```

```expect
2:1 yaml/document-start
```

```fixed
# settings
---
a: 1
```
