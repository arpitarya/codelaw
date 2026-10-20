# py/unused-variable — cases

### case: read in a nested format spec

```py
def f(x):
    width = 3
    return f"{x:{width}d}"
```

```expect
(none)
```

### case: underscore names are deliberate

```py
def f():
    _ = compute()
    _unused = 1
```

```expect
(none)
```

### case: locals() reads everything

```py
def f():
    x = 1
    return locals()
```

```expect
(none)
```

### case: nonlocal use

```py
def f():
    x = 1
    def g():
        nonlocal x
        x = 2
    return g
```

```expect
(none)
```

### case: module level is not checked

```py
x = 1
```

```expect
(none)
```

### case: only the last assignment is reported

```py
def f():
    x = 1
    x = 2
```

```expect
3:5 py/unused-variable
```

```fixed
def f():
    pass
```

### case: unused exception name

```py
def f():
    try:
        pass
    except ValueError as err:
        pass
```

```expect
4:26 py/unused-variable
```

```fixed
def f():
    try:
        pass
    except ValueError:
        pass
```

### case: loop variables are B007's business

```py
def f():
    for i in range(3):
        pass
```

```expect
(none)
```

### case: augmented assignment reads

```py
def f():
    t = 1
    t += 1
```

```expect
(none)
```

### case: unpacking a call result is not reported

```py
def f(s):
    name, j = s.split()
    return j
```

```expect
(none)
```

### case: unpacking a tuple literal is

```py
def f():
    head, tail = 1, 2
    return tail
```

```expect
2:5 py/unused-variable
```

### case: module-level except name

```py
try:
    pass
except ValueError as err:
    pass
```

```expect
3:22 py/unused-variable
```

```fixed
try:
    pass
except ValueError:
    pass
```
