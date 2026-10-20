# html/form-dup-name — cases

### case: array names may repeat

```html
<form>
  <input type="text" name="tags[]">
  <input type="text" name="tags[]">
</form>
```

```expect
(none)
```

### case: separate forms are separate

```html
<form><input type="text" name="q"></form>
<form><input type="text" name="q"></form>
```

```expect
(none)
```

### case: disabled controls are not submitted

```html
<form>
  <input type="text" name="q">
  <input type="text" name="q" disabled>
</form>
```

```expect
(none)
```
