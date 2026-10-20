# py/no-print — cases

### case: adds the import

```py
def run():
    print("x")
```

```expect
2:5 py/no-print
```

```fixed
import logging
def run():
    logging.getLogger(__name__).info("%s", "x")
```

### case: several arguments: no edit

```py
print("a", 1)
```

```expect
1:1 py/no-print
```

### case: a local print function

```py
def print(x):
    pass
print(1)
```

```expect
(none)
```
