# py/max-lines-per-function — cases

### case: a function

```py options={"max":2}
def a():
    b()
    c()
```

```expect
1:5 py/max-lines-per-function
```

### case: an async function

```py options={"max":2}
async def a():
    b()
    c()
```

```expect
1:11 py/max-lines-per-function
```

### case: a method

```py options={"max":2}
class A:
    def run(self):
        b()
        c()
```

```expect
2:9 py/max-lines-per-function
```

### case: a decorated function starts at its def

```py options={"max":2}
@deco
@other
def a():
    b()
    c()
```

```expect
3:5 py/max-lines-per-function
```

### case: a lambda is not a function here

```py options={"max":0}
f = lambda x: x
```

```expect
(none)
```

### case: blank lines and comments are not counted

```py options={"max":2}
def a():

    # note
    b()
```

```expect
(none)
```

### case: blank lines counted

```py options={"max":3,"skip_blank_lines":false}
def a():

    b()
    c()
```

```expect
1:5 py/max-lines-per-function
```

### case: docstring lines are not counted by default

```py options={"max":2}
def a():
    """Doc
    more
    lines."""
    b()
```

```expect
(none)
```

### case: docstring lines counted

```py options={"max":2,"skip_docstrings":false}
def a():
    """Doc
    more."""
    b()
```

```expect
1:5 py/max-lines-per-function
```

### case: an inner function counts toward the outer one

```py options={"max":4}
def outer():
    def inner():
        b()
    inner()
    c()
```

```expect
1:5 py/max-lines-per-function
```

### case: an inner function left out of the outer one

```py options={"max":3,"include_nested":false}
def outer():
    def inner():
        b()
        c()
    inner()
```

```expect
(none)
```

### case: ignore_names

```py options={"max":2,"ignore_names":"^test_"}
def test_a():
    b()
    c()
def other():
    b()
    c()
```

```expect
4:5 py/max-lines-per-function
```

### case: a function at the limit passes

```py options={"max":3}
def a():
    b()
    c()
```

```expect
(none)
```

### case: an invalid ignore_names is reported once, and the function still measured

```py options={"max":2,"ignore_names":"("}
def a():
    b()
    c()
```

```expect
1:1 py/max-lines-per-function
1:5 py/max-lines-per-function
```
