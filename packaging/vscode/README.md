# codelaw for VS Code

Diagnostics, quick fixes, fix-on-save and formatting from
[codelaw](https://github.com/arpitarya/codelaw) — one linter and formatter for
JavaScript, TypeScript, Python, JSON, YAML, TOML, Markdown, CSS, SCSS and HTML.
The extension holds no lint logic: it starts `codelaw lsp` and adds commands, so
the editor shows exactly what `codelaw check` reports
([SR-EDITOR](https://github.com/arpitarya/codelaw/blob/main/records/0114_editor.md)).

**Which codelaw runs:** `codelaw.path`, else the workspace's
`node_modules/.bin/codelaw`, else `.venv/bin/codelaw` (or beside
`python.defaultInterpreterPath`), else the binary bundled in this extension's
platform package. The project's own codelaw wins, so the editor agrees with CI.

**Commands:** Fix all · Format · Explain rule · Restart server · Show output ·
Copy debug info.
**Settings:** `codelaw.enable`, `codelaw.path`, `codelaw.fixOnSave`,
`codelaw.formatOnSave`, `codelaw.logLevel`, `codelaw.trace.server`.

## Troubleshooting

- **Where the log is:** run *codelaw: Show output* (or click the status-bar item).
  It starts with the extension, VS Code and OS versions, every codelaw binary
  it tried and why it used the one it did, and the server's version.
- **A file shows nothing:** the log says why, once per file: no covered
  language, excluded, outside `include`, or a config that does not load.
- **More detail:** set `codelaw.logLevel` to `debug` (no restart needed) to add
  one line per check with the rule count, diagnostic count and milliseconds.
  `codelaw.trace.server` is the raw protocol, for protocol bugs only.
- **Reporting a problem:** run *codelaw: Copy debug info* and paste the result
  in [an issue](https://github.com/arpitarya/codelaw/issues). It holds versions,
  the binary path, settings and the last 200 log lines; the log never holds
  your source text or environment variables.

## Developing

```bash
npm ci && npm test     # compiles and tests the binary lookup and the logging helpers
```

`version` is the one version (L6 decision 9): `scripts/version.py --set` writes
it here and in `package-lock.json`, and `--check` fails when either drifts.
`scripts/vsix.py build` makes the eight platform `.vsix` files, each with its
`bin/codelaw`; `scripts/vsix.py check` checks them. Every link in this file is
absolute: it is the Marketplace and Open VSX page, where a relative link breaks.
