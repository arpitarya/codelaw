# py/useless-expression — cases

### case: arithmetic on literals

```py
def f():
    1 / 0
```

```expect
2:5 py/useless-expression
```

### case: docstrings and ellipsis

```py
def f():
    """Doc."""
    ...
```

```expect
(none)
```

### case: calls have effects

```py
def f(a):
    a.b()
```

```expect
(none)
```

### case: a bare name

```py
def f(a):
    a
```

```expect
2:5 py/useless-expression
```

### case: a tuple of attributes

```py
import termios
termios.tcgetattr, termios.tcsetattr
```

```expect
2:1 py/useless-expression
```
