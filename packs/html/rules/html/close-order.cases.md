# html/close-order — cases

### case: a pre closes an open paragraph, so its end tag is stray

```html
<p>
Text
<pre>code</pre>
more
</p>
```

```expect
5:1 html/close-order
```

### case: optional end tags are not unclosed

```html
<table>
  <tr><td>1<td>2
</table>
```

```expect
(none)
```

### case: a missing start tag

```html
<head><title>t</title></head>
</html>
```

```expect
2:1 html/close-order
```
