# toml/deps-sorted — edge cases

### case: optional dependencies

```toml
[project.optional-dependencies]
test = ["pytest", "Coverage[toml]"]
```

```expect
2:1 toml/deps-sorted
```

```fixed
[project.optional-dependencies]
test = ["Coverage[toml]", "pytest"]
```

### case: names normalised

```toml
[project]
dependencies = ["typing_extensions", "Typing-Inspect"]
```

```expect
(none)
```
