# sec-py/cookie-flags — cases

### case: no flags

```py
resp.set_cookie("a", "b")
```

```expect
1:1 sec-py/cookie-flags
```

### case: all flags

```py
resp.set_cookie("a", "b", secure=True, httponly=True, samesite="Lax")
```

```expect
(none)
```

### case: samesite None

```py
resp.set_cookie("a", "b", secure=True, httponly=True, samesite=None)
```

```expect
1:1 sec-py/cookie-flags
```

### case: one flag missing

```py
resp.set_cookie("a", "b", secure=True, samesite="Lax")
```

```expect
1:1 sec-py/cookie-flags
```

### case: flags from variables

```py
resp.set_cookie("a", "b", secure=secure, httponly=httponly, samesite=samesite)
```

```expect
(none)
```

### case: secure=False

```py
resp.set_cookie("a", "b", secure=False, httponly=True, samesite="Lax")
```

```expect
1:1 sec-py/cookie-flags
```
