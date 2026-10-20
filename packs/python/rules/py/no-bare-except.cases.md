# py/no-bare-except — cases

### case: a handler that re-raises

```py
try:
    run()
except:
    cleanup()
    raise
```

```expect
(none)
```

### case: a conditional re-raise still swallows

```py
try:
    run()
except:
    if strict:
        raise
```

```expect
3:1 py/no-bare-except
```

```fixed
try:
    run()
except Exception:
    if strict:
        raise
```
