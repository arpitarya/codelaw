# sec-py/jinja-autoescape — cases

### case: autoescape=False

```py
env = Environment(autoescape=False)
```

```expect
1:7 sec-py/jinja-autoescape
```

### case: no autoescape given

```py
env = Environment(loader=loader)
```

```expect
1:7 sec-py/jinja-autoescape
```

### case: select_autoescape

```py
env = Environment(autoescape=select_autoescape())
```

```expect
(none)
```

### case: SSTI

```py
def v(request):
    return render_template_string(request.args["t"])
```

```expect
2:12 sec-py/jinja-autoescape
```

### case: from_string

```py
env.from_string(text)
```

```expect
1:1 sec-py/jinja-autoescape
```

### case: a literal template

```py
render_template_string("<p>{{ x }}</p>", x=x)
```

```expect
(none)
```
