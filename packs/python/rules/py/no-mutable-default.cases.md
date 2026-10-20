# py/no-mutable-default — cases

### case: after a docstring

```py
def f(x={}):
    """Doc."""
    return x
```

```expect
1:9 py/no-mutable-default
```

```fixed
def f(x=None):
    """Doc."""
    if x is None:
        x = {}
    return x
```

### case: constructor calls

```py
def f(a=set(), b=dict(k=1), c=list()):
    pass
```

```expect
1:9 py/no-mutable-default
1:18 py/no-mutable-default
1:31 py/no-mutable-default
```

### case: a default that reads the outer name: no edit

```py
def outer(lnum):
    def reader(lnum=[lnum]):
        return lnum
    return reader
```

```expect
2:21 py/no-mutable-default
```

### case: immutable defaults

```py
def f(a=(), b=frozenset(), c=None, d=""):
    pass
```

```expect
(none)
```
