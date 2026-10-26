# sec-js/no-jquery-html-sink — cases

### case: a selector variable is not a tag

```js
$(selector).hide();
```

```expect
(none)
```

### case: html() with no argument reads

```js
const h = $("#a").html();
```

```expect
(none)
```

### case: jQuery(tag)

```js
jQuery("<div>" + label + "</div>");
```

```expect
1:1 sec-js/no-jquery-html-sink
```

### case: html with a value

```js
$("#a").html(userHtml);
```

```expect
1:1 sec-js/no-jquery-html-sink
```
