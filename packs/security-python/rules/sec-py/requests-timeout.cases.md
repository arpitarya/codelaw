# sec-py/requests-timeout — cases

### case: no timeout

```py
requests.get(url)
```

```expect
1:1 sec-py/requests-timeout
```

### case: with timeout

```py
requests.get(url, timeout=3)
```

```expect
(none)
```

### case: a session is not matched

```py
session.get(url)
```

```expect
(none)
```

### case: post with data

```py
requests.post(url, json=body)
```

```expect
1:1 sec-py/requests-timeout
```
