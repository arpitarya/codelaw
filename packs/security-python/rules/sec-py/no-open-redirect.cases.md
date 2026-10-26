# sec-py/no-open-redirect — cases

### case: a bound request value

```py
def v(request):
    n = request.args.get("next")
    return redirect(n)
```

```expect
3:12 sec-py/no-open-redirect
```

### case: Django

```py
def v(request):
    return HttpResponseRedirect(request.GET["next"])
```

```expect
2:12 sec-py/no-open-redirect
```

### case: a route

```py
def v(request):
    return redirect("/home")
```

```expect
(none)
```
