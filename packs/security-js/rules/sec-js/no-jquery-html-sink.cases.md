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

### case: extend: the project function passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml } from "@acme/security";
$(el).html(cleanHtml(x));
```

```expect
(none)
```

### case: extend: the built-in DOMPurify still passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import DOMPurify from "dompurify";
$(el).html(DOMPurify.sanitize(x));
```

```expect
(none)
```

### case: replace: the project function passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}],"sanitizers_mode":"replace"}
import { cleanHtml } from "@acme/security";
$(el).html(cleanHtml(x));
```

```expect
(none)
```

### case: replace: DOMPurify is reported

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}],"sanitizers_mode":"replace"}
import DOMPurify from "dompurify";
$(el).html(DOMPurify.sanitize(x));
```

```expect
2:1 sec-js/no-jquery-html-sink
```

### case: an aliased import passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml as clean } from "@acme/security";
$(el).html(clean(x));
```

```expect
(none)
```

### case: a same-named local function is reported

```js
function sanitizeHtml(x) { return x; }
$(el).html(sanitizeHtml(x));
```

```expect
2:1 sec-js/no-jquery-html-sink
```

### case: a function of the same name from another module is reported

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml } from "left-pad";
$(el).html(cleanHtml(x));
```

```expect
2:1 sec-js/no-jquery-html-sink
```

### case: the global DOMPurify passes

```js
$(el).html(DOMPurify.sanitize(x));
```

```expect
(none)
```

### case: the built-in sanitize-html import passes

```js
import sanitizeHtml from "sanitize-html";
$(el).html(sanitizeHtml(x));
```

```expect
(none)
```
