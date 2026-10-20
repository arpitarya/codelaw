# html/no-implicit-close — cases

### case: a paragraph closed by a following block

```html
<p>One
<div>Two</div>
```

```expect
1:1 html/no-implicit-close
```

```fixed
<p>One</p>
<div>Two</div>
```

### case: table cells closed by the next row

```html
<table>
  <tr><td>1
  <tr><td>2</td></tr>
</table>
```

```expect
2:3 html/no-implicit-close
2:7 html/no-implicit-close
```

```fixed
<table>
  <tr><td>1</td></tr>
  <tr><td>2</td></tr>
</table>
```
