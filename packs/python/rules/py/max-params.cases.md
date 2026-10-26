# py/max-params — cases

### case: a function

```py options={"max":2}
def a(x, y, z):
    pass
```

```expect
1:5 py/max-params
```

### case: an async function

```py options={"max":2}
async def a(x, y, z):
    pass
```

```expect
1:11 py/max-params
```

### case: self is not counted

```py options={"max":2}
class A:
    def run(self, x, y):
        pass
```

```expect
(none)
```

### case: cls is not counted

```py options={"max":2}
class A:
    @classmethod
    def run(cls, x, y):
        pass
```

```expect
(none)
```

### case: self, counted

```py options={"max":2,"ignore_self_cls":false}
class A:
    def run(self, x, y):
        pass
```

```expect
2:9 py/max-params
```

### case: self outside a class is a parameter

```py options={"max":2}
def run(self, x, y):
    pass
```

```expect
1:5 py/max-params
```

### case: annotations and defaults

```py options={"max":2}
def a(x: int, y: str = 'a', z: int = 1):
    pass
```

```expect
1:5 py/max-params
```

### case: keyword-only parameters count

```py options={"max":2}
def a(x, *, y, z):
    pass
```

```expect
1:5 py/max-params
```

### case: positional-only parameters count

```py options={"max":2}
def a(x, y, /, z):
    pass
```

```expect
1:5 py/max-params
```

### case: args and kwargs are not counted

```py options={"max":2}
def a(x, y, *args, **kwargs):
    pass
```

```expect
(none)
```

### case: args and kwargs, counted

```py options={"max":2,"count_varargs":true}
def a(x, y, *args, **kwargs):
    pass
```

```expect
1:5 py/max-params
```

### case: ignore_names

```py options={"max":2,"ignore_names":"^_"}
def a(x, _y, _z):
    pass
```

```expect
(none)
```

### case: a function at the limit passes

```py options={"max":2}
def a(x, y):
    pass
```

```expect
(none)
```

### case: max_positional

```py options={"max":10,"max_positional":2}
def a(x, y, z):
    pass
```

```expect
1:5 py/max-params
```

### case: max_positional ignores keyword-only parameters

```py options={"max":10,"max_positional":2}
def a(x, y, *, z, w):
    pass
```

```expect
(none)
```

### case: max_positional ignores parameters after args

```py options={"max":10,"max_positional":2}
def a(x, y, *args, z):
    pass
```

```expect
(none)
```

### case: both caps

```py options={"max":2,"max_positional":2}
def a(x, y, z):
    pass
```

```expect
1:5 py/max-params
1:5 py/max-params
```

### case: an override is skipped

```py options={"max":2}
class A(B):
    @override
    def run(self, a, b, c):
        pass
```

```expect
(none)
```

### case: a typing.override is skipped

```py options={"max":2}
class A(B):
    @typing.override
    def run(self, a, b, c):
        pass
```

```expect
(none)
```

### case: an override, counted

```py options={"max":2,"ignore_overrides":false}
class A(B):
    @override
    def run(self, a, b, c):
        pass
```

```expect
3:9 py/max-params
```

### case: an overload stub is skipped

```py options={"max":2}
@overload
def a(x, y, z): ...
```

```expect
(none)
```

### case: an overload stub, counted

```py options={"max":2,"ignore_declarations":false}
@typing.overload
def a(x, y, z): ...
```

```expect
2:5 py/max-params
```

### case: a lambda is not a function here

```py options={"max":0}
f = lambda x, y, z: 1
```

```expect
(none)
```

### case: an invalid ignore_names is reported once, and the function still measured

```py options={"max":2,"ignore_names":"("}
def a(x, y, z):
    pass
```

```expect
1:1 py/max-params
1:5 py/max-params
```
