# sec-py/no-nosql-injection — cases

### case: a body used whole

```py
def f(request):
    return db.users.find(request.json)
```

```expect
2:12 sec-py/no-nosql-injection
```

### case: a built filter

```py
def f(request):
    return db.users.find({"name": str(request.json["name"])})
```

```expect
(none)
```

### case: $where

```py
def f(request):
    q = {"$where": f"this.a == {request.args['a']}"}
```

```expect
2:10 sec-py/no-nosql-injection
```
