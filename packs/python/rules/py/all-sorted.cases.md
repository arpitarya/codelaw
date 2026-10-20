# py/all-sorted — cases

### case: multi-line list

```py
__all__ = [
    "b",
    "a",
]
```

```expect
1:11 py/all-sorted
```

```fixed
__all__ = [
    "a",
    "b",
]
```

### case: several items per line

```py
__all__ = ["b", "a",
           "c"]
```

```expect
1:11 py/all-sorted
```

```fixed
__all__ = ["a",
           "b",
           "c"]
```

### case: case-sensitive within a group

```py
__all__ = ["LWPCookieJar", "LoadError"]
```

```expect
(none)
```

### case: augmented and bare tuples

```py
__all__ = "b", "a"
if x:
    __all__ += ["d", "c"]
```

```expect
1:11 py/all-sorted
3:16 py/all-sorted
```

### case: comments block the fix

```py
__all__ = [
    "b",  # keep
    "a",
]
```

```expect
1:11 py/all-sorted
```
