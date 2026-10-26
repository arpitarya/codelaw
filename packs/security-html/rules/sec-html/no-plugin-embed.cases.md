# sec-html/no-plugin-embed — cases

### case: object, embed and applet

```html
<object data="a.swf"></object>
<embed src="a.swf">
<applet code="A"></applet>
```

```expect
1:1 sec-html/no-plugin-embed
2:1 sec-html/no-plugin-embed
3:1 sec-html/no-plugin-embed
```

### case: an object that holds an image

```html
<object type="image/svg+xml" data="a.svg"></object>
```

```expect
(none)
```
