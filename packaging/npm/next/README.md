# @codelaw/next

codelaw's findings in `next dev` and `next build`. It runs the `codelaw` binary
and never lints itself.

```js
// next.config.js
import { withCodelaw } from "@codelaw/next";
export default withCodelaw({ /* your config */ });
```

- Under **webpack**: the client compile carries `@codelaw/webpack-plugin`'s behaviour (errors and warnings in the dev overlay and terminal; `next build` fails on an error).
- Under **Turbopack**, which has no webpack-plugin API: `next dev` runs the watcher beside the server and prints findings to the terminal; `next build` runs one `codelaw check --profile ci` first and fails on an error.

Options: `profile`, `buildProfile`, `failOnError`, `paths`, `binary`.
