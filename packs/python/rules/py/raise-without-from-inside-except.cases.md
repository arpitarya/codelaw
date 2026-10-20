# py/raise-without-from-inside-except — cases

### case: raise inside a nested try

```py
try:
    run()
except ValueError:
    try:
        raise KeyError()
    except KeyError:
        pass
```

```expect
(none)
```

### case: re-raising

```py
try:
    run()
except ValueError as e:
    raise e
except KeyError:
    raise
```

```expect
(none)
```

### case: in a nested function

```py
try:
    run()
except ValueError:
    def f():
        raise TypeError()
```

```expect
(none)
```
