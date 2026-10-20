# html/element-permitted-content — cases

### case: obsolete elements are left to html/deprecated

```html
<p>Use <tt>make</tt> here.</p>
```

```expect
(none)
```

### case: a transparent link passes its content through

```html
<p><a href="/a"><div>Block</div></a></p>
```

```expect
1:17 html/element-permitted-content
```

### case: the svg content model is not HTML

```html
<p><svg><g><rect width="1" height="1"></rect></g></svg></p>
```

```expect
(none)
```

### case: a list nested directly in a list

```html
<ul>
  <li>One</li>
  <ul><li>Two</li></ul>
</ul>
```

```expect
3:3 html/element-permitted-content
```

### case: content of a template is checked against its own parent

```html
<template><td>1</td></template>
```

```expect
(none)
```
