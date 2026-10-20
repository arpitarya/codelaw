# toml/pyproject-requires-python — edge cases

### case: no [project] table

```toml
[tool.poetry]
name = "app"
```

```expect
(none)
```

### case: declared dynamic

```toml
[project]
name = "app"
dynamic = ["requires-python", "version"]
```

```expect
(none)
```
