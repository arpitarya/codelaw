# sec-py/no-reflected-xss — cases

### case: a direct value

```py
def v(request):
    return HttpResponse(request.GET["n"])
```

```expect
2:12 sec-py/no-reflected-xss
```

### case: a returned string

```py
def v():
    return "<p>" + request.args["n"] + "</p>"
```

```expect
2:5 sec-py/no-reflected-xss
```

### case: escaped

```py
def v(request):
    return HttpResponse(escape(request.GET["n"]))
```

```expect
(none)
```

### case: a constant

```py
def v(request):
    return HttpResponse("ok")
```

```expect
(none)
```
