---
type: Handoff
name: W-74
description: "Arpit, 2026-10-26: the VS Code extension and `codelaw lsp` log too little to debug a problem from — make the output channel say what is running, what it decided for each file, and why anything failed or did nothing."
item: W-74
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-74 — Editor logs you can debug from

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The extension's first lines say what is running: extension, VS Code and OS versions, every binary it tried and why it picked one, the server's version | "which codelaw is this?" answered without asking | ⏳ |
| 2 | The server says which config and project each file got, and when and why it reloads | "why these rules here?" answered from the log | ⏳ |
| 3 | Every "nothing happened" says why: no project, file excluded, language not covered, project failed to load, no fix, no formatter | silent files stop being a mystery | ⏳ |
| 4 | Every failure is logged with its context: failed requests, server crash and exit code, Explain rule failing, a project that won't load | an error is in the log, not only in a popup | ⏳ |
| 5 | Timestamps and log levels on one channel, a `codelaw.logLevel` setting; `debug` adds timings and counts per check | quiet by default, detailed when asked | ⏳ |
| 6 | A *Copy debug info* command: versions, binary, config path and the last log lines, ready to paste into an issue | one click from a problem to a bug report | ⏳ |

**Arpit, 2026-10-26:** *"The VS Code extension doesn't have very good output or logging. Create a work item for better logs so that at least it can be debugged if there is any issue."*

**Model:** Sonnet: bounded TypeScript and Rust plumbing in two small files, no design forks the record doesn't already settle.

## Goal

Someone whose editor shows the wrong thing — or nothing — can open *codelaw: Show output*, read what happened, and either fix it or paste it into an issue. Nobody has to attach a debugger or turn on raw LSP tracing to find out which binary ran or why a file has no diagnostics.

## What it logs today (2026-10-26, read from the code)

| where | what is logged | what is missing |
|---|---|---|
| `packaging/vscode/src/extension.ts` `start()` | one line, `codelaw: <path> (<source>)`, or the no-binary tooltip | the candidates tried and why each was skipped; the server's version (`serverInfo.version` is sent but never printed); extension and VS Code versions; a `client.start()` failure is not caught |
| `extension.ts` *Explain rule* | stdout or stderr of `codelaw explain` | returns silently when there is no binary |
| `crates/codelaw-lsp/src/lib.rs` `project_for()` | a config error (`logMessage` ERROR); project warnings (WARNING) | which config and project a file resolved to; a project that fails to load goes to `showMessage` — a popup — and not into the log |
| `lib.rs` `check()` | nothing | a file with no project, no entry or no language gets an empty diagnostic list silently; no timing, no count |
| `lib.rs` `request()` | nothing | a failed request goes back as an `InvalidParams` response only; nothing in the log names the method, file or error |
| `lib.rs` `reload()` | nothing | which file change triggered it |
| `codelaw.trace.server` | raw LSP JSON | it is a protocol dump, not a log — too noisy to read and off by default |

## Design pointers (not rulings)

- **Extension:** `vscode.window.createOutputChannel("codelaw", { log: true })` (a `LogOutputChannel`, VS Code ≥ 1.74; the extension needs ^1.82) gives timestamps and levels and follows *Developer: Set Log Level* — no dependency (L11).
- **Server:** keep `window/logMessage` (the `log()` helper exists); add the `Log` level for debug lines, gated by an `initializationOptions.logLevel` the extension passes. Nothing on stdout — it is the transport.
- **A project that fails to load:** both the popup (once per project) and the log line.
- **Never log source text.** Paths, rule ids, counts and timings only.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · startup | first lines of the channel: extension version, VS Code version, OS/arch, each binary candidate with its verdict, the chosen one, `serverInfo.version`, workspace folders | ⏳ |
| 2 · per file | at `info`: the config path and project root a file resolved to, once per project; at `debug`: per check — file, language, rule count, diagnostic count, milliseconds | ⏳ |
| 3 · why nothing | each of: no config above the file · file outside every project's `files` · excluded · language not covered · project failed to load — logs one line naming the file and the reason (once per file per reload, not per keystroke) | ⏳ |
| 4 · failures | failed requests (method, file, error); `client.start()` rejecting; server exit (code, last stderr lines) and restarts; *Explain rule* with no binary; *Fix all* / *Format* returning nothing at `debug` | ⏳ |
| 5 · levels | `codelaw.logLevel`: `error` · `warn` · `info` (default) · `debug`; changing it applies without a restart; `codelaw.trace.server` unchanged | ⏳ |
| 6 · debug info | *codelaw: Copy debug info* puts versions, binary path and source, config path(s), settings and the last 200 log lines on the clipboard | ⏳ |
| 7 · tests | `codelaw-lsp` unit tests: a file outside any project logs its reason; a bad config logs and shows; a failed request logs. Extension: `binary.test.ts` covers the candidate trace | ⏳ |
| 8 · perf | `info` adds no per-keystroke lines; `debug` off by default; L8 budgets for the LSP still met | ⏳ |
| 9 · docs | SR-EDITOR updated; `packaging/vscode/README.md` gains *Troubleshooting* (where the log is, how to raise the level, what to paste in an issue); `IMPLEMENTATION.md`; CHANGELOG | ⏳ |

## Blockers

None.

## Record affected

[SR-EDITOR](../../records/0114_editor.md) decision 4 (the settings list gains `codelaw.logLevel`; the commands list gains *Copy debug info*), and a new decision there on what the server and extension log, at which level, and that source text is never logged.

## Hazards

- **stdout is the LSP transport.** A stray `println!` in the server breaks the editor; log only through `window/logMessage` or stderr.
- **Log volume.** `didChange` fires per keystroke; anything per check belongs at `debug`, and "why nothing" lines are de-duplicated per file per reload.
- **Privacy.** A log gets pasted into public issues: no source text, no environment variables; paths are fine.
- **L11.** No logging library in the extension or the server unless the decider ratifies it; the VS Code API and `lsp-types` already cover this.
- **L9 for the building session:** check `target/` against the 5 GB budget ([SR-LAW-9](../../records/0011_LAW-9-machine-budget.md)) before building.

## Reference

- [`packaging/vscode/src/extension.ts`](../../packaging/vscode/src/extension.ts) · [`packaging/vscode/src/binary.ts`](../../packaging/vscode/src/binary.ts)
- [`crates/codelaw-lsp/src/lib.rs`](../../crates/codelaw-lsp/src/lib.rs) — `log()`, `project_for()`, `check()`, `request()`, `reload()`
- [SR-EDITOR](../../records/0114_editor.md)
- VS Code `LogOutputChannel` — `window.createOutputChannel(name, { log: true })`
