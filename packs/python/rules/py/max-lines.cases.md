# py/max-lines — cases

### case: blank lines and comments are not counted

```py options={"max":2}
a = 1

# one
b = 2
```

```expect
(none)
```

### case: blank lines counted

```py options={"max":3,"skip_blank_lines":false}
a = 1


b = 2
```

```expect
4:1 py/max-lines
```

### case: a line of code and a comment is code

```py options={"max":1}
a = 1  # note
b = 2  # note
```

```expect
2:1 py/max-lines
```

### case: docstrings are counted by default

```py options={"max":3}
"""Module
doc."""
import os
x = os
```

```expect
4:1 py/max-lines
```

### case: docstrings skipped

```py options={"max":3,"skip_docstrings":true}
"""Module
doc."""
import os
x = os
```

```expect
(none)
```

### case: a function docstring

```py options={"max":2,"skip_docstrings":true}
def f():
    """Doc
    more."""
    return 1
```

```expect
(none)
```

### case: a string statement that is not a docstring counts

```py options={"max":2,"skip_docstrings":true}
x = 1
"""not a docstring
at all"""
y = 2
```

```expect
3:1 py/max-lines
```

### case: the import block

```py options={"max":1,"skip_imports":true}
import os
from sys import path
use(os, path)
```

```expect
(none)
```

### case: imports counted by default

```py options={"max":1}
import os
from sys import path
use(os, path)
```

```expect
2:1 py/max-lines
```

### case: an empty module

```py options={"max":0}

```

```expect
(none)
```
