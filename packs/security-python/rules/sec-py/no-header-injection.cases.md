# sec-py/no-header-injection — cases

### case: a request value

```py
def v(request):
    resp.headers["X-A"] = request.args["a"]
```

```expect
2:5 sec-py/no-header-injection
```

### case: a constant

```py
def v(request):
    resp.headers["X-A"] = "1"
```

```expect
(none)
```

### case: Django

```py
def v(request):
    response["X-A"] = request.GET["a"]
```

```expect
2:5 sec-py/no-header-injection
```

### case: a dict is not matched

```py
def v(request):
    d["x"] = request.GET["a"]
```

```expect
(none)
```
