# sec-py/no-ssrf — cases

### case: a request URL

```py
def f(request):
    return requests.get(request.args["u"], timeout=5)
```

```expect
2:12 sec-py/no-ssrf
```

### case: an f-string

```py
def f(request):
    return requests.get(f"https://{request.GET['h']}/x", timeout=5)
```

```expect
2:12 sec-py/no-ssrf
```

### case: a constant

```py
def f(request):
    return requests.get("https://api.example.com", timeout=5)
```

```expect
(none)
```
