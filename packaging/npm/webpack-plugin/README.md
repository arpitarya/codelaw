# @codelaw/webpack-plugin

codelaw's findings as webpack errors and warnings, for `webpack`, `webpack serve`
and **Rspack** (the same plugin, unchanged). It runs the `codelaw` binary and
never lints itself, so a finding is the one `codelaw check` prints.

```js
// webpack.config.js  (rspack.config.js is the same)
import CodelawPlugin from "@codelaw/webpack-plugin";
export default { plugins: [new CodelawPlugin()] };
```

- Watching (`webpack serve`, `--watch`): `codelaw check --watch` on the `local` profile; a save that changes the findings recompiles once with them. Set `devServer.client.overlay.warnings: false` to keep warnings out of the overlay.
- A build: one `codelaw check --profile ci`; an error is a compilation error (`failOnError: false` makes it a warning).
- Closing the watcher stops codelaw.

Options: `profile`, `buildProfile`, `failOnError`, `paths`, `binary`.
