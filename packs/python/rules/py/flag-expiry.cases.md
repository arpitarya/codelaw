# Cases: py/flag-expiry

### case: a method wrapper and an enum member, across modules

```python options={"presets":["launchdarkly"]}
# app/flags.py
import enum
class Flag(str, enum.Enum):
    NEW = "new-checkout"
class Flags:
    def on(self, key, default=False):
        return client.variation(key, None, default)
# app/views.py
from app.flags import Flag, Flags
flags = Flags()
ENABLED = flags.on(Flag.NEW.value)
```

```expect
app/views.py:3:11 py/flag-expiry
```

### case: a decided flag in an `elif` chain keeps the rest of the chain

```python options={"accessors":["is_on($NAME)"]}
# .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
keep = "off"
# app/a.py
def f(x):
    if is_on("beta"):
        return 1
    elif x:
        return 2
    else:
        return 3
```

```expect
app/a.py:2:8 py/flag-expiry
```

```fixed
# .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
keep = "off"
# app/a.py
def f(x):
    if x:
        return 2
    else:
        return 3
```

### case: a decided flag in a conditional expression

```python options={"accessors":["is_on($NAME)"]}
# .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
keep = "on"
# app/a.py
LIMIT = 10 if is_on("beta") else 5
```

```expect
app/a.py:1:15 py/flag-expiry
```

```fixed
# .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
keep = "on"
# app/a.py
LIMIT = 10
```

### case: an f-string name is info, uncertain

```python options={"accessors":["is_on($NAME)"]}
# app/a.py
def f(k):
    return is_on(f"flag-{k}")
```

```expect
app/a.py:2:12 py/flag-expiry
```
