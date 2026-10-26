# sec-js/no-dangerously-set-inner-html — cases

### case: in JSX

```jsx
const a = <div dangerouslySetInnerHTML={{ __html: html }} />;
```

```expect
1:43 sec-js/no-dangerously-set-inner-html
```

### case: a literal

```js
const p = { dangerouslySetInnerHTML: { __html: "<b>x</b>" } };
```

```expect
(none)
```

### case: an unsanitized call result

```js
const p = { dangerouslySetInnerHTML: { __html: render(md) } };
```

```expect
1:40 sec-js/no-dangerously-set-inner-html
```

### case: extend: the project function passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml } from "@acme/security";
const p = { dangerouslySetInnerHTML: { __html: cleanHtml(x) } };
```

```expect
(none)
```

### case: extend: the built-in DOMPurify still passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import DOMPurify from "dompurify";
const p = { dangerouslySetInnerHTML: { __html: DOMPurify.sanitize(x) } };
```

```expect
(none)
```

### case: replace: the project function passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}],"sanitizers_mode":"replace"}
import { cleanHtml } from "@acme/security";
const p = { dangerouslySetInnerHTML: { __html: cleanHtml(x) } };
```

```expect
(none)
```

### case: replace: DOMPurify is reported

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}],"sanitizers_mode":"replace"}
import DOMPurify from "dompurify";
const p = { dangerouslySetInnerHTML: { __html: DOMPurify.sanitize(x) } };
```

```expect
2:40 sec-js/no-dangerously-set-inner-html
```

### case: an aliased import passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml as clean } from "@acme/security";
const p = { dangerouslySetInnerHTML: { __html: clean(x) } };
```

```expect
(none)
```

### case: a same-named local function is reported

```js
function sanitizeHtml(x) { return x; }
const p = { dangerouslySetInnerHTML: { __html: sanitizeHtml(x) } };
```

```expect
2:40 sec-js/no-dangerously-set-inner-html
```

### case: a function of the same name from another module is reported

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml } from "left-pad";
const p = { dangerouslySetInnerHTML: { __html: cleanHtml(x) } };
```

```expect
2:40 sec-js/no-dangerously-set-inner-html
```

### case: the global DOMPurify passes

```js
const p = { dangerouslySetInnerHTML: { __html: DOMPurify.sanitize(x) } };
```

```expect
(none)
```

### case: the built-in sanitize-html import passes

```js
import sanitizeHtml from "sanitize-html";
const p = { dangerouslySetInnerHTML: { __html: sanitizeHtml(x) } };
```

```expect
(none)
```

### case: an instance made by the module's factory passes

```js
import createDOMPurify from "dompurify";
const DOMPurify = createDOMPurify(window);
const p = { dangerouslySetInnerHTML: { __html: DOMPurify.sanitize(html) } };
```

```expect
(none)
```
