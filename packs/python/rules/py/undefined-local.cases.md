# py/undefined-local — cases

### case: annotations of a nested function

```py
from typing import Any, Iterator


def _unique(fn):
    def unique(*args: Any, **kw: Any) -> Iterator[Any]:
        return fn(*args, **kw)
    return unique
```

```expect
(none)
```

### case: builtin annotation in a nested function

```py
def outer():
    def parse_line(line: str) -> str:
        return line
    return parse_line
```

```expect
(none)
```

### case: read in the right-hand side

```py
x = 1
def f():
    x = x + 1
```

```expect
3:9 py/undefined-local
```

### case: assigned first

```py
x = 1
def f():
    x = 2
    return x
```

```expect
(none)
```

### case: global declaration

```py
x = 1
def f():
    global x
    x += 1
```

```expect
(none)
```

### case: no outer name

```py
def f():
    print(y)
    y = 1
```

```expect
(none)
```
