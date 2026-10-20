# html/valid-autocomplete — cases

### case: a section and a hint before the field

```html
<input type="text" name="a" autocomplete="section-ship shipping address-line1">
```

```expect
(none)
```

### case: a multi-line field belongs on a textarea

```html
<input type="text" name="a" autocomplete="street-address">
<textarea name="b" autocomplete="street-address"></textarea>
```

```expect
1:43 html/valid-autocomplete
```

### case: on and off are not combined with fields

```html
<input type="text" name="a" autocomplete="off">
```

```expect
(none)
```

### case: form takes only on or off

```html
<form autocomplete="email"></form>
```

```expect
1:21 html/valid-autocomplete
```
