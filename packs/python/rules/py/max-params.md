---
id: py/max-params
title: "A function takes at most a maximum number of parameters"
languages: [python]
fix: manual
fix_reason: "grouping parameters changes the function's signature and every call to it; which belong together is the author's call"
stability: preview
options:
  max: { type: "integer", default: 5, description: "the most parameters a function may take" }
  max_positional: { type: "integer", description: "a separate cap on positional parameters, those before `*` or `*args` (Ruff PLR0917); none by default" }
  count_varargs: { type: "boolean", default: false, description: "count `*args` and `**kwargs`" }
  ignore_self_cls: { type: "boolean", default: true, description: "`self` and `cls`, the first parameter of a method, do not count" }
  ignore_names: { type: "string", default: "^_", description: "a regular expression; parameters named like this do not count" }
  ignore_overrides: { type: "boolean", default: true, description: "skip a method decorated `@override` or `@typing.override`: its signature is not yours" }
  ignore_declarations: { type: "boolean", default: true, description: "skip an `@overload` stub" }
ported_from: [ruff/PLR0913, ruff/PLR0917, pylint/too-many-arguments]
since: 0.3.0
---

A function (`def` or `async def`, a method included) takes at most `max` parameters (5 by default, as Pylint and
Ruff). `self` and `cls` are not counted, nor are `*args` and `**kwargs` unless `count_varargs` is `true`, nor a
parameter whose name matches `ignore_names` (`"^_"` by default). A keyword-only parameter counts. `max_positional`
adds a second cap on the parameters a caller can pass by position, those before `*` or `*args`. A method decorated
`@override` and an `@overload` stub are skipped, since you do not own that signature; set `ignore_overrides` or
`ignore_declarations` to `false` to count them. The rule reports on the function's name:
`create_order takes 6 parameters (max 5)`.

**Why:** A call with many positional arguments is easy to get in the wrong order, and every added parameter is a change to every
caller. Grouping related parameters in one object, or making them keyword-only, names them at the call site.

```codelaw
native: py/max-params
```

## How to fix

1. Group the parameters that belong together into a dataclass or a `TypedDict`: `create_order(Order(customer, items, coupon, address, note, gift))`.
2. Or make the rarely used ones keyword-only with a bare `*`, and set `max_positional` to cap how many a caller passes by position.
3. If the signature is not yours to change, exempt it: an `@override` and an `@overload` stub already are.

## Examples

### Python

#### Don't

```py
def create_order(customer, items, coupon, address, note, gift):
    return (customer, items, coupon, address, note, gift)
```

#### Do

```py
def create_order(order):
    return order
```
