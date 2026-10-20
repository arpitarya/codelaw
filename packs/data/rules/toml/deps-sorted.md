---
id: toml/deps-sorted
title: "pyproject.toml dependencies are sorted by name"
languages: [toml]
files: ["**/pyproject.toml"]
fix: auto
stability: preview
since: 0.1.0
---

The requirement lists in `[project].dependencies` and in each `[project.optional-dependencies]` extra are sorted by package name. Names compare the way pip compares them: case, `-`, `_` and `.` don't matter.

**Why:** A sorted list is quicker to scan, shows a duplicate at once and gives smaller diffs; installers don't depend on the order.

```codelaw
native: toml/deps-sorted
```

## Examples

### TOML

#### Don't

```toml path=pyproject.toml
[project]
name = "app"
dependencies = [
    "requests>=2.31",
    "click>=8",
    "attrs",
]
```

#### Do

```toml path=pyproject.toml
[project]
name = "app"
dependencies = [
    "attrs",
    "click>=8",
    "requests>=2.31",
]
```

## Notes

A list with comments in it is reported without a fix, because the comments would stay on their lines while the requirements move.
