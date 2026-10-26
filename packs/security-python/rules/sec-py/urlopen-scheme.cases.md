# sec-py/urlopen-scheme — cases

### case: a name

```py
urlopen(url)
```

```expect
1:1 sec-py/urlopen-scheme
```

### case: an https literal

```py
urlopen("https://example.com")
```

```expect
(none)
```

### case: qualified

```py
urllib.request.urlopen(u, timeout=5)
```

```expect
1:1 sec-py/urlopen-scheme
```

### case: an f-string with a fixed prefix

```py
urlopen(f"https://example.com/{x}")
```

```expect
(none)
```
