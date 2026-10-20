# py/ambiguous-variable-name — cases

### case: lambda parameters are not checked

```py
f = lambda l: l
```

```expect
(none)
```

### case: loop and with targets

```py
for l in items:
    pass
with open(p) as I:
    pass
```

```expect
1:5 py/ambiguous-variable-name
3:17 py/ambiguous-variable-name
```

### case: import alias

```py
import numpy as l
```

```expect
1:17 py/ambiguous-variable-name
```

### case: augmented assignment

```py
def f(l):
    l -= 1
```

```expect
1:7 py/ambiguous-variable-name
2:5 py/ambiguous-variable-name
```
