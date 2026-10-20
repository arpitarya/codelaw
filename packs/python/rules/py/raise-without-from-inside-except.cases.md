# py/raise-without-from-inside-except — cases

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
