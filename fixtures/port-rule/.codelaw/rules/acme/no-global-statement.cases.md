# acme/no-global-statement — cases

Written fresh from the check's documented behaviour; Pylint is GPL-2.0, so nothing here is taken from
its tests or its documentation's examples.

### case: in a method

```py
cache = None


class Loader:
    def load(self):
        global cache
        cache = {}
```

```expect
6:9 acme/no-global-statement
```

### case: in a nested function

```py
def outer():
    def inner():
        global flag
        flag = True
    inner()
```

```expect
3:9 acme/no-global-statement
```

### case: at module level, another check's

```py
global settings
settings = {}
```

```expect
(none)
```

### case: nonlocal is not global

```py
def outer():
    total = 0

    def add(n):
        nonlocal total
        total += n
    return add
```

```expect
(none)
```

### case: reading a module variable needs no global

```py
LIMIT = 10


def over(n):
    return n > LIMIT
```

```expect
(none)
```
