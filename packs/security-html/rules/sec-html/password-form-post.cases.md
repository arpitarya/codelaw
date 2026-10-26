# sec-html/password-form-post — cases

### case: get is replaced

```html
<form method="GET"><input type="password"></form>
```

```expect
1:1 sec-html/password-form-post
```

```fixed
<form method="post"><input type="password"></form>
```

### case: no method means get

```html
<form action="/x"><input type="password"></form>
```

```expect
1:1 sec-html/password-form-post
```

```fixed
<form method="post" action="/x"><input type="password"></form>
```

### case: a framework handles the submit

```html
<form (ngSubmit)="go()"><input type="password"></form>
```

```expect
(none)
```

### case: an http action

```html
<form method="post" action="http://a.example/l"><input type="password"></form>
```

```expect
1:21 sec-html/password-form-post
```

```fixed
<form method="post" action="https://a.example/l"><input type="password"></form>
```

### case: no password, no finding

```html
<form method="get"><input type="search" name="q"></form>
```

```expect
(none)
```
