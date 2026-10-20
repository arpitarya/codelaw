# py/f-string-missing-placeholders — cases

### case: escaped braces become plain braces

```py
x = f"{{literal}}"
```

```expect
1:5 py/f-string-missing-placeholders
```

```fixed
x = "{literal}"
```

### case: one part of a concatenation has a placeholder

```py
x = f"a" f"{b}"
```

```expect
(none)
```

### case: all parts lack placeholders

```py
x = (f"a" "b")
```

```expect
1:6 py/f-string-missing-placeholders
```

```fixed
x = ("a" "b")
```

### case: nested f-string in a format spec

```py
x = f"{v:{w}}"
```

```expect
(none)
```
