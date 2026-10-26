# sec-html/no-unsafe-template-interpolation — cases

### case: each syntax

```html
<p>{{ x|safe }}</p>
{% autoescape off %}
<%- user %>
```

```expect
1:4 sec-html/no-unsafe-template-interpolation
2:1 sec-html/no-unsafe-template-interpolation
3:1 sec-html/no-unsafe-template-interpolation
```

### case: a literal string marked safe

```html
<p>{{ "<b>"|safe }}</p>
```

```expect
(none)
```

### case: EJS include and ERB trim marks

```html
<%- include('a') %><%- if x -%>
```

```expect
(none)
```

### case: a raw block is not triple-stache

```html
{{{{raw}}}}
```

```expect
(none)
```
