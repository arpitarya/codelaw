# py/lambda-assignment — cases

### case: in a class body

```py
class C:
    m = lambda self: 1
```

```expect
2:5 py/lambda-assignment
```

```fixed
class C:
    def m(self):
        return 1
```

### case: tuple targets are fine

```py
a, b = lambda: 0, 1
```

```expect
(none)
```

### case: lambda as an argument

```py
items.sort(key=lambda x: x[1])
```

```expect
(none)
```

### case: attribute targets are fine

```py
obj.handler = lambda: None
```

```expect
(none)
```
