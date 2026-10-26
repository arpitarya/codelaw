# sec-py/no-format-string-injection — cases

### case: a request template

```py
def g(request):
    return request.GET["t"].format(user)
```

```expect
2:12 sec-py/no-format-string-injection
```

### case: a literal template

```py
def g(request):
    return "Hello {}".format(request.GET["n"])
```

```expect
(none)
```

### case: format_map

```py
def g(request):
    t = request.args["t"]
    return t.format_map(d)
```

```expect
3:12 sec-py/no-format-string-injection
```
