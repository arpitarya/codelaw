# py/complexity — cases

### case: a function over the limit is reported on its name

```py options={"max":2}
def f(a, b):
    if a:
        return 1
    if b:
        return 2
    return 3
```

```expect
1:5 py/complexity
```

### case: at the limit is fine

```py options={"max":3}
def f(a, b):
    if a:
        return 1
    if b:
        return 2
    return 3
```

```expect
```

### case: elif, and, or and the conditional expression

```py options={"max":4}
def f(a, b):
    if a and b:
        return 1
    elif a or b:
        return 2
    return 3 if a else 4
```

```expect
1:5 py/complexity
```

### case: loops, except and comprehension clauses

```py options={"max":4}
def f(xs):
    for x in xs:
        while x:
            try:
                x()
            except ValueError:
                pass
    return [y for y in xs if y]
```

```expect
1:5 py/complexity
```

### case: match cases count, the wildcard does not

```py options={"max":2}
def f(a):
    match a:
        case 1:
            return 1
        case 2:
            return 2
        case _:
            return 3
```

```expect
1:5 py/complexity
```

### case: a nested def is counted on its own

```py options={"max":2}
def outer(a):
    def inner(b):
        if a:
            return 1
        if b:
            return 2
        return 3
    return inner
```

```expect
2:9 py/complexity
```

### case: ignore_names exempts a function

```py options={"max":1,"ignore_names":"^route"}
def route_request(a):
    if a:
        return 1
    return 2
```

```expect
```
