# @codelaw/vite-plugin

codelaw's findings while `vite` runs. It never lints itself: it runs the
`codelaw` binary (`codelaw check --watch --format json`), so a finding is the
same one `codelaw check` prints.

```js
// vite.config.js
import codelaw from "@codelaw/vite-plugin";
export default { plugins: [codelaw()] };
```

- `vite` (dev): the `local` profile; errors go to Vite's overlay and the terminal, warnings to the terminal only.
- `vite build`: one `codelaw check --profile ci`; an error fails the build (`failOnError: false` makes it a warning).
- Stopping the dev server stops the watcher.

Options: `profile`, `buildProfile`, `failOnError`, `paths`, `binary`. The binary is
`CODELAW_BINARY`, else the project's own `codelaw` package, else `codelaw` on PATH.
