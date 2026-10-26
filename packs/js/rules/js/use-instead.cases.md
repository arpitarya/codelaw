# js/use-instead — cases

### case: a call is reported

```js options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true,"allow_in":["src/lib/http.js"]},"process.env":{"use":"env","from":"@acme/env"}}}
fetch("/a");
```

```expect
1:1 js/use-instead
```

### case: a member read matches by prefix

```js options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true,"allow_in":["src/lib/http.js"]},"process.env":{"use":"env","from":"@acme/env"}}}
const k = process.env.API_URL;
```

```expect
1:11 js/use-instead
```

### case: a local shadow is ignored

```js options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true,"allow_in":["src/lib/http.js"]},"process.env":{"use":"env","from":"@acme/env"}}}
function f(fetch) { return fetch("/a"); }
```

```expect
(none)
```

### case: a property key and a declaration are ignored

```js options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true,"allow_in":["src/lib/http.js"]},"process.env":{"use":"env","from":"@acme/env"}}}
const o = { fetch: 1 };
function fetch2() {}
o.fetch();
```

```expect
(none)
```

### case: an import already present is not added again

```js options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true,"allow_in":["src/lib/http.js"]},"process.env":{"use":"env","from":"@acme/env"}}}
import { apiFetch } from "@acme/http";
fetch("/a");
```

```expect
2:1 js/use-instead
```

### case: a name that merely starts alike is not a prefix

```js options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true,"allow_in":["src/lib/http.js"]},"process.env":{"use":"env","from":"@acme/env"}}}
const x = process.environment;
```

```expect
(none)
```

### case: a non-drop-in entry is reported without a fix

```js options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true,"allow_in":["src/lib/http.js"]},"process.env":{"use":"env","from":"@acme/env"}}}
const k = process.env.API_URL;
```

```expect
1:11 js/use-instead
```

### case: an imported root is matched

```js options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true,"allow_in":["src/lib/http.js"]},"process.env":{"use":"env","from":"@acme/env"}}}
import fetch from "node-fetch";
fetch("/a");
```

```expect
2:1 js/use-instead
```
