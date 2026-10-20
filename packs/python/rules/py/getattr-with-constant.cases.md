# py/getattr-with-constant — cases

### case: with a default it stays

```py
name = getattr(user, "name", None)
```

```expect
(none)
```

### case: private names are mangled

```py
x = getattr(self, "__secret")
```

```expect
(none)
```

### case: not an identifier

```py
x = getattr(obj, "a-b")
```

```expect
(none)
```
