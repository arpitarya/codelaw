# py/max-depth — cases

### case: a statement nested past the limit is reported on its first line

```py options={"max":2}
def f(a):
    if a:
        while a:
            if a:
                a()
```

```expect
4:13 py/max-depth
```

### case: at the limit is fine

```py options={"max":3}
def f(a):
    if a:
        while a:
            if a:
                a()
```

```expect
```

### case: elif and else add no depth

```py options={"max":1}
def f(a):
    if a == 1:
        a()
    elif a == 2:
        a()
    else:
        a()
```

```expect
```

### case: try, with and match count

```py options={"max":2}
def f(a):
    try:
        with a:
            match a:
                case 1:
                    pass
    except ValueError:
        pass
```

```expect
4:13 py/max-depth
```

### case: a def starts again at zero

```py options={"max":1}
if a:
    def g(x):
        if x:
            x()
```

```expect
```
