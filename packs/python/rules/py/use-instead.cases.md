# py/use-instead — cases

### case: an attribute call is reported

```py options={"replacements":{"requests.get":{"use":"http_get","from":"acme.http","drop_in":true},"os.environ":{"use":"env","from":"acme.env"}}}
import requests
requests.get("u")
```

```expect
2:1 py/use-instead
```

### case: an alias is read as written, so it is not matched

```py options={"replacements":{"requests.get":{"use":"http_get","from":"acme.http","drop_in":true},"os.environ":{"use":"env","from":"acme.env"}}}
import requests as r
r.get("u")
```

```expect
(none)
```

### case: a prefix read is reported

```py options={"replacements":{"requests.get":{"use":"http_get","from":"acme.http","drop_in":true},"os.environ":{"use":"env","from":"acme.env"}}}
import os
k = os.environ.get("A")
```

```expect
2:5 py/use-instead
```

### case: a local shadow is ignored

```py options={"replacements":{"requests.get":{"use":"http_get","from":"acme.http","drop_in":true},"os.environ":{"use":"env","from":"acme.env"}}}
def f(requests):
    return requests.get("u")
```

```expect
(none)
```

### case: a keyword argument name is ignored

```py options={"replacements":{"requests.get":{"use":"http_get","from":"acme.http","drop_in":true},"os.environ":{"use":"env","from":"acme.env"}}}
f(requests=1)
```

```expect
(none)
```
