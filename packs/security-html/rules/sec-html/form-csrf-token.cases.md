# sec-html/form-csrf-token — cases

### case: no token

```html
<form method="post" action="/a"><input name="x"></form>
```

```expect
1:1 sec-html/form-csrf-token
```

### case: a hidden field

```html
<form method="post"><input type="hidden" name="csrfmiddlewaretoken" value="x"></form>
```

```expect
(none)
```

### case: a template tag that writes the field

```html
<form method="post">{% csrf_token %}</form>
```

```expect
(none)
```

### case: a form that posts to another site

```html
<form method="post" action="https://pay.example/"><input name="x"></form>
```

```expect
(none)
```

### case: GET needs none

```html
<form method="get"><input name="q"></form>
```

```expect
(none)
```
