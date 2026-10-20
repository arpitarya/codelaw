# py/unintentional-type-annotation — cases

### case: self attributes are declarations

```py
class C:
    def __init__(self):
        self.x: int
```

```expect
(none)
```
