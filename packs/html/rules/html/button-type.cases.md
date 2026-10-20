# html/button-type — cases

### case: a button inside a select is the picker button

```html
<select>
  <button>Pick</button>
  <option>a</option>
</select>
```

```expect
(none)
```

### case: attributes stay after the inserted type

```html
<button class="x">Go</button>
```

```expect
1:1 html/button-type
```

```fixed
<button type="button" class="x">Go</button>
```
