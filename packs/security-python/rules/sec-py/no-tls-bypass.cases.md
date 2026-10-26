# sec-py/no-tls-bypass — cases

### case: verify=False

```py
requests.post(url, data=d, verify=False, timeout=5)
```

```expect
1:1 sec-py/no-tls-bypass
```

### case: verify with a bundle

```py
requests.get(url, verify="/etc/ca.pem")
```

```expect
(none)
```

### case: unverified context

```py
ctx = ssl._create_unverified_context()
```

```expect
1:7 sec-py/no-tls-bypass
```

### case: check_hostname

```py
ctx.check_hostname = False
```

```expect
1:1 sec-py/no-tls-bypass
```

### case: CERT_NONE

```py
ctx.verify_mode = ssl.CERT_NONE
```

```expect
1:1 sec-py/no-tls-bypass
```

### case: old protocol

```py
ctx = ssl.SSLContext(ssl.PROTOCOL_TLSv1)
```

```expect
1:22 sec-py/no-tls-bypass
```

### case: default context

```py
ctx = ssl.create_default_context()
```

```expect
(none)
```
