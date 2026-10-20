# py/reuse-of-groupby-generator — cases

### case: separate branches

```py
import itertools
for k, g in itertools.groupby(rows):
    if k:
        a = list(g)
    else:
        b = list(g)
```

```expect
(none)
```
