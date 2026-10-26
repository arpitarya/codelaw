# sec-py/no-path-traversal — cases

### case: a request value

```py
def v(request):
    return send_file(request.args["f"])
```

```expect
2:12 sec-py/no-path-traversal
```

### case: a bound name

```py
def v(request):
    f = request.GET.get("f")
    return open(f)
```

```expect
3:12 sec-py/no-path-traversal
```

### case: a constant

```py
def v(request):
    return send_file("report.pdf")
```

```expect
(none)
```

### case: a parameter is not followed

```py
def v(name):
    return open(name)
```

```expect
(none)
```
