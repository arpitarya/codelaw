# sec-py/no-csrf-exempt — cases

### case: exempt

```py
@csrf_exempt
def v(request):
    pass
```

```expect
1:1 sec-py/no-csrf-exempt
```

### case: qualified

```py
@csrf.csrf_exempt
def v(request):
    pass
```

```expect
1:1 sec-py/no-csrf-exempt
```

### case: another decorator

```py
@login_required
def v(request):
    pass
```

```expect
(none)
```

### case: csrf_protect

```py
@csrf_protect
def v(request):
    pass
```

```expect
(none)
```
