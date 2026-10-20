# codelaw for VS Code

Diagnostics, quick fixes and (once the formatter ships) formatting from
`codelaw lsp`. The extension holds no lint logic; it starts the server and adds
commands ([SR-EDITOR](../../records/0114_editor.md)).

**Which codelaw runs:** `codelaw.path`, else the workspace's
`node_modules/.bin/codelaw`, else `.venv/bin/codelaw` (or beside
`python.defaultInterpreterPath`), else the binary bundled in this extension's
platform package. The project's own codelaw wins, so the editor agrees with CI.

**Commands:** Fix all · Format · Explain rule · Restart server · Show output.
**Settings:** `codelaw.enable`, `codelaw.path`, `codelaw.fixOnSave`,
`codelaw.formatOnSave`, `codelaw.trace.server`.

```bash
npm ci && npm test     # compiles and tests the binary lookup
```

`version` is `0.0.0` here; the release stamps the one version (L6 decision 9).
