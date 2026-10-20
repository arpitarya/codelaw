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

### case: PEP 695 type parameters reach annotations, bases and methods

```py
class Base[T]:
    pass


def first[T](xs: list[T]) -> T:
    return xs[0]


class Box[T](Base[T]):
    def get(self) -> T:
        item: T = self.item
        return item


type Pair[K] = tuple[K, K]
p: Pair[int] = (1, 2)
```

```expect
(none)
```

### case: a type parameter is not visible outside its declaration

```py
def first[T](xs: list[T], default=T) -> T:
    return xs[0]


x: T
```

```expect
1:35 py/undefined-name
5:4 py/undefined-name
```
