# sec-js/express-hardening — cases

### case: disabled in the file

```js
const app = express();
app.disable("x-powered-by");
```

```expect
(none)
```
