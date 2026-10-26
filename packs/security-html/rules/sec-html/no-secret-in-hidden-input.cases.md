# sec-html/no-secret-in-hidden-input — cases

### case: a literal key

```html
<input type="hidden" name="api_key" value="sk_live_1">
```

```expect
1:37 sec-html/no-secret-in-hidden-input
```

### case: csrf tokens are fine

```html
<input type="hidden" name="csrf_token" value="abc"><input type="hidden" name="_token" value="abc">
```

```expect
(none)
```

### case: a value filled in at render time or empty

```html
<input type="hidden" name="api_key" value="{{ key }}"><input type="hidden" name="api_key" value="">
```

```expect
(none)
```
