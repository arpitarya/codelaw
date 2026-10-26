# sec-js/no-unsanitized-html — cases

### case: insertAdjacentHTML has no mechanical fix

```js
box.insertAdjacentHTML("beforeend", html);
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: a sanitizer call is not flagged

```js
element.innerHTML = DOMPurify.sanitize(comment.body);
```

```expect
(none)
```

### case: a literal is fine

```js
element.innerHTML = "<b>hi</b>";
```

```expect
(none)
```

### case: outerHTML

```js
element.outerHTML = html;
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: document.write

```js
document.write(markup);
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: innerHTML +=

```js
el.innerHTML += chunk;
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: createContextualFragment

```js
range.createContextualFragment(html);
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: setHTMLUnsafe

```js
el.setHTMLUnsafe(html);
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: srcdoc

```js
frame.srcdoc = html;
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: parseFromString

```js
new DOMParser().parseFromString(html, "text/html");
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: extend: the project function passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml } from "@acme/security";
element.innerHTML = cleanHtml(x);
```

```expect
(none)
```

### case: extend: the built-in DOMPurify still passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import DOMPurify from "dompurify";
element.innerHTML = DOMPurify.sanitize(x);
```

```expect
(none)
```

### case: replace: the project function passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}],"sanitizers_mode":"replace"}
import { cleanHtml } from "@acme/security";
element.innerHTML = cleanHtml(x);
```

```expect
(none)
```

### case: replace: DOMPurify is reported

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}],"sanitizers_mode":"replace"}
import DOMPurify from "dompurify";
element.innerHTML = DOMPurify.sanitize(x);
```

```expect
2:1 sec-js/no-unsanitized-html
```

### case: an aliased import passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml as clean } from "@acme/security";
element.innerHTML = clean(x);
```

```expect
(none)
```

### case: a same-named local function is reported

```js
function sanitizeHtml(x) { return x; }
element.innerHTML = sanitizeHtml(x);
```

```expect
2:1 sec-js/no-unsanitized-html
```

### case: a function of the same name from another module is reported

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml } from "left-pad";
element.innerHTML = cleanHtml(x);
```

```expect
2:1 sec-js/no-unsanitized-html
```

### case: the global DOMPurify passes

```js
element.innerHTML = DOMPurify.sanitize(x);
```

```expect
(none)
```

### case: the built-in sanitize-html import passes

```js
import sanitizeHtml from "sanitize-html";
element.innerHTML = sanitizeHtml(x);
```

```expect
(none)
```
