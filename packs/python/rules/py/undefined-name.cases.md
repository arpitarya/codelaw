# py/undefined-name — cases

### case: class body sees its own names

```py
class C:
    x = 1
    y = x + 1
```

```expect
(none)
```

### case: methods don't see class names

```py
class C:
    x = 1
    def m(self):
        return x
```

```expect
4:16 py/undefined-name
```

### case: __class__ in a method

```py
class C:
    def m(self):
        return __class__
```

```expect
(none)
```

### case: guarded by NameError

```py
try:
    unicode
except NameError:
    unicode = str
```

```expect
(none)
```

### case: star import makes everything possible

```py
from os.path import *
print(join)
```

```expect
(none)
```

### case: comprehension variable

```py
values = [x for x in range(3)]
```

```expect
(none)
```

### case: module dunders

```py
print(__file__, __name__, __doc__)
```

```expect
(none)
```

### case: del of an undefined name

```py
del missing
```

```expect
1:5 py/undefined-name
```
