# sec-py/no-regex-injection — cases

### case: a request pattern

```py
def f(request, t):
    return re.search(request.GET["q"], t)
```

```expect
2:12 sec-py/no-regex-injection
```

### case: escaped

```py
def f(request, t):
    return re.search(re.escape(request.GET["q"]), t)
```

```expect
(none)
```

### case: a literal

```py
re.compile("a+")
```

```expect
(none)
```
