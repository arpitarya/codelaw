# sec-py/no-dynamic-import — cases

### case: getattr

```py
def v(request):
    n = request.GET["a"]
    return getattr(handlers, n)()
```

```expect
3:12 sec-py/no-dynamic-import
```

### case: import_module

```py
def v(request):
    return importlib.import_module(request.args["m"])
```

```expect
2:12 sec-py/no-dynamic-import
```

### case: a constant

```py
def v(request):
    return getattr(handlers, "run")
```

```expect
(none)
```
