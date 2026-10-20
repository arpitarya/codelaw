---
id: py/abstract-base-class-without-abstract-method
title: "An abstract base class has an abstract method"
languages: [python]
fix: manual
fix_reason: "which methods subclasses must provide is a design decision"
stability: preview
ported_from: [ruff/B024]
since: 0.1.0
---

A class that inherits only from `ABC` (or uses `ABCMeta`) declares at least one `@abstractmethod`.

**Why:** Without one, nothing stops the base class being instantiated, and readers can't tell what subclasses must provide.

```codelaw
native: py/abstract-base-class-without-abstract-method
```

## How to fix

1. Mark the methods subclasses must override with `@abstractmethod`, or
2. drop `ABC` if the class isn't meant to be abstract.

## Examples

### Python

#### Don't

```py
from abc import ABC


class Shape(ABC):
    def area(self):
        return 0
```

#### Do

```py
from abc import ABC, abstractmethod


class Shape(ABC):
    @abstractmethod
    def area(self):
        return 0
```
