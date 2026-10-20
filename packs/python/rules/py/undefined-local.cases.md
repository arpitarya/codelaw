# py/undefined-local — cases

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
