# sec-py/no-silent-except — cases

### case: bare except

```py
try:
    f()
except:
    pass
```

```expect
3:1 sec-py/no-silent-except
```

### case: Exception

```py
try:
    f()
except Exception:
    pass
```

```expect
3:1 sec-py/no-silent-except
```

### case: a specific exception

```py
try:
    f()
except KeyError:
    pass
```

```expect
(none)
```

### case: logged

```py
try:
    f()
except Exception:
    log.exception("x")
```

```expect
(none)
```
