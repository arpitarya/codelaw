# html/no-raw-characters — cases

### case: a doctype tree-sitter cannot parse is not raw text

```html
<!DOCTYPE>
<p>Hi</p>
```

```expect
(none)
```

### case: an unterminated reference is a raw ampersand

```html
<p>a&nbsp b</p>
```

```expect
1:5 html/no-raw-characters
```

```fixed
<p>a&amp;nbsp b</p>
```

### case: = in an unquoted attribute

```html
<a href=/x?a=1>x</a>
```

```expect
1:13 html/no-raw-characters
```

```fixed
<a href=/x?a&equals;1>x</a>
```

### case: quoted attribute values may hold anything

```html
<a href="/x?a=1&b=2">x</a>
```

```expect
(none)
```

### case: script content is not text

```html
<script>if (a < b && c) {}</script>
```

```expect
(none)
```

### case: template tags are left alone

```html
<p><%= user.name %></p>
```

```expect
(none)
```

### case: text tree-sitter cannot parse, with multi-byte characters

```html
<p>中文 & 日本語</p>
```

```expect
1:7 html/no-raw-characters
```

```fixed
<p>中文 &amp; 日本語</p>
```
