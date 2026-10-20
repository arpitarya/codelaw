# py/zip-without-explicit-strict — cases

### case: infinite iterator

```py
import itertools
pairs = zip(names, itertools.count())
```

```expect
(none)
```

### case: starred arguments

```py
rows = zip(*columns)
```

```expect
1:8 py/zip-without-explicit-strict
```

```fixed
rows = zip(*columns, strict=False)
```
