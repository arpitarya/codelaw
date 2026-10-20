# py/redefined-while-unused — cases

### case: try/except alternatives

```py
try:
    import json
except ImportError:
    import simplejson as json
```

```expect
(none)
```

### case: if/else alternatives

```py
if fast:
    def run(): pass
else:
    def run(): pass
```

```expect
(none)
```

### case: property setter

```py
class C:
    @property
    def p(self): pass
    @p.setter
    def p(self, v): pass
```

```expect
(none)
```

### case: overloads

```py
from typing import overload
@overload
def f(a: int): ...
@overload
def f(a: str): ...
def f(a): pass
```

```expect
(none)
```

### case: import replaced by assignment

```py
import os
os = 1
```

```expect
2:1 py/redefined-while-unused
```

### case: plain variables may be reassigned

```py
x = 1
x = 2
```

```expect
(none)
```

### case: used in between

```py
import os
print(os.name)
import os
```

```expect
(none)
```

### case: methods in a class

```py
class C:
    def m(self): pass
    def m(self): pass
```

```expect
3:9 py/redefined-while-unused
```

### case: two submodules of one package

```py
import urllib.parse, urllib.request
```

```expect
(none)
```

### case: global rebinding from a function

```py
xmlrpclib = None
def f():
    global xmlrpclib
    import xmlrpc.client as xmlrpclib
```

```expect
(none)
```

### case: optional accelerator

```py
def cmp_to_key(f): pass
try:
    from _functools import cmp_to_key
except ImportError:
    pass
```

```expect
(none)
```

### case: right-hand side reads the old binding

```py
def _coro(): pass
_coro = _coro()
```

```expect
(none)
```
