# sec-py/no-cleartext-http — cases

### case: plain http

```py
requests.get("http://example.com", timeout=5)
```

```expect
1:1 sec-py/no-cleartext-http
```

### case: https

```py
requests.get("https://example.com")
```

```expect
(none)
```

### case: localhost

```py
requests.get("http://localhost:8000/x")
```

```expect
(none)
```

### case: an f-string

```py
httpx.post(f"http://example.com/{p}/x")
```

```expect
1:1 sec-py/no-cleartext-http
```
