# sec-py/no-mark-safe — cases

### case: a concatenation

```py
mark_safe("<b>" + name + "</b>")
```

```expect
1:1 sec-py/no-mark-safe
```

### case: an f-string

```py
mark_safe(f"<b>{name}</b>")
```

```expect
1:1 sec-py/no-mark-safe
```

### case: a literal

```py
mark_safe("<br>")
```

```expect
(none)
```

### case: Markup

```py
Markup(user_text)
```

```expect
1:1 sec-py/no-mark-safe
```

### case: format_html

```py
format_html("<b>{}</b>", name)
```

```expect
(none)
```
