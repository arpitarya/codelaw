# sec-py/no-mass-assignment — cases

### case: a splat

```py
def v(request):
    return Post.objects.create(**request.POST)
```

```expect
2:12 sec-py/no-mass-assignment
```

### case: a constructor

```py
def v(request):
    return Post(**request.data)
```

```expect
2:12 sec-py/no-mass-assignment
```

### case: __all__

```py
class Meta:
    fields = "__all__"
```

```expect
2:5 sec-py/no-mass-assignment
```

### case: a list

```py
class Meta:
    fields = ["title"]
```

```expect
(none)
```

### case: a plain splat

```py
Post.objects.create(**defaults)
```

```expect
(none)
```
