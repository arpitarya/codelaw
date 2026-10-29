# py/skipped-test — cases

### case: every skip decorator

```py
import pytest, unittest

@pytest.mark.skip
def test_a(): pass

@pytest.mark.skipif(True, reason="r")
def test_b(): pass

@pytest.mark.xfail
def test_c(): pass

class T(unittest.TestCase):
    @unittest.skip("later")
    def test_d(self): pass

    @unittest.skipUnless(True, "r")
    def test_e(self): pass
```

```expect
3:2 py/skipped-test
6:2 py/skipped-test
9:2 py/skipped-test
13:6 py/skipped-test
16:6 py/skipped-test
```

### case: a comment above the decorators gives both

```py
# SKIP(ABC-1, 2027-06-01): flaky on CI
@pytest.mark.skip
def test_a(): pass
```

```expect
(none)
```

### case: the framework's reason and the comment's expiry

```py
# SKIP(2027-06-01)
@pytest.mark.skipif(True, reason="no GPU")
def test_a(): pass
```

```expect
(none)
```

### case: a module-level mark

```py
pytestmark = pytest.mark.skip(reason="rewrite")
```

```expect
1:14 py/skipped-test
```

### case: other marks are not skips

```py
@pytest.mark.parametrize("a", [1])
def test_a(a): pass
```

```expect
(none)
```
