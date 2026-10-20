# py/reuse-of-groupby-generator — cases

### case: single use as a generator source

```py
import itertools
for k, g in itertools.groupby(items):
    out.extend(e for e in g)
```

```expect
(none)
```

### case: use in an inner for of a comprehension

```py
import itertools
for k, g in itertools.groupby(items):
    out = [x for y in ys for x in g]
```

```expect
3:35 py/reuse-of-groupby-generator
```

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
