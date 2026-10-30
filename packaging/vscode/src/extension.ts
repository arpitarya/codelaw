// The thin VS Code extension (SR-EDITOR decision 4): it finds a codelaw
// binary, starts `codelaw lsp`, and adds commands and a status-bar item.
// It holds no lint or format logic; every verdict comes from the server.
// How a finding looks — badges, gutter glyphs, chips, the card's actions,
// the sidebar, the counts — is drawn from the server's `data` (W-137,
// SR-EDITOR decision 8; decorations.ts, sidebar.ts, status.ts, actions.ts).

import * as fs from "node:fs";
import { execFile, spawn } from "node:child_process";
import * as vscode from "vscode";
import { LanguageClient, LanguageClientOptions, ServerOptions, State } from "vscode-languageclient/node";
import { Found, Tried, traceBinary } from "./binary";
import { fromClientLine, Level, LEVELS, parseLevel, renderDebugInfo, Ring } from "./logging";
import { registerActions } from "./actions";
import { decorateHover, Painter, restyle, settings, Store } from "./decorations";
import { CardArgs } from "./findings";
import { openManifests, Sidebar } from "./sidebar";
import { Counts, Tabs } from "./status";

const FIX_ALL = "source.fixAll.codelaw";
const LANGUAGES = [
  "javascript",
  "javascriptreact",
  "typescript",
  "typescriptreact",
  "python",
  "json",
  "jsonc",
  "yaml",
  "toml",
  "markdown",
  "css",
  "scss",
  "html",
];
// The plain-text config files a finding may sit in, by name (SR-LAW-3
// decision 10): opened under any language id, so matched by pattern. Keep
// in step with `TEXT_CONFIG_FILES` in crates/codelaw-core/src/text_config.rs.
const TEXT_CONFIG = "**/{.nvmrc,.node-version,.python-version,.tool-versions,Dockerfile,CODEOWNERS,yarn.lock}";

let client: LanguageClient | undefined;
let binary: Found | undefined;
let output: vscode.LogOutputChannel;
let status: vscode.StatusBarItem;
let level: Level = "info";
let tried: Tried[] = [];
let serverVersion: string | undefined;
const store = new Store();
let painter: Painter | undefined;
/** The last lines logged, for *Copy debug info*. */
const recent = new Ring(200);
/** The last lines the server wrote to stderr, for its exit report. */
const stderrTail = new Ring(10);

/**
 * One log line, kept only when `codelaw.logLevel` allows it (SR-EDITOR
 * decision 7). Paths, rule ids, counts and timings only; never source text.
 * A debug line goes out at the channel's info level behind `[debug]`,
 * because the channel itself hides debug unless VS Code's own log level is
 * raised too.
 */
function log(at: Level, message: string): void {
  if (LEVELS.indexOf(at) > LEVELS.indexOf(level)) return;
  recent.push(`${new Date().toISOString()} [${at}] ${message}`);
  if (at === "error") output.error(message);
  else if (at === "warn") output.warn(message);
  else if (at === "info") output.info(message);
  else output.info(`[debug] ${message}`);
}

/**
 * What the language client writes to its output channel: its own
 * `[Level - time] text` lines and the server's `logMessage`s. They are
 * parsed and sent through `log`, so one channel has one set of timestamps.
 */
function clientChannel(): vscode.OutputChannel {
  return {
    name: "codelaw",
    append: (value: string) => log("debug", value),
    appendLine: (value: string) => {
      const { level: at, text } = fromClientLine(value);
      log(at, text);
    },
    replace: () => undefined,
    clear: () => undefined,
    show: ((): void => output.show()) as vscode.OutputChannel["show"],
    hide: () => output.hide(),
    dispose: () => undefined,
  };
}

function config() {
  return vscode.workspace.getConfiguration("codelaw");
}

async function start(context: vscode.ExtensionContext): Promise<void> {
  const folders = (vscode.workspace.workspaceFolders ?? []).map((f) => f.uri.fsPath);
  log("info", `codelaw extension ${context.extension.packageJSON.version} on VS Code ${vscode.version}, ${process.platform} ${process.arch}`);
  log("info", `workspace folders: ${folders.join(", ") || "none"}`);
  if (!config().get<boolean>("enable", true)) {
    status.text = "codelaw: off";
    status.show();
    log("info", "codelaw.enable is false: not starting the server");
    return;
  }
  const trace = traceBinary(locateLookup(context));
  tried = trace.tried;
  for (const t of tried) log("info", `binary candidate ${t.source}: ${t.path} - ${t.verdict}`);
  binary = trace.found;
  if (!binary) {
    status.text = "$(warning) codelaw";
    status.tooltip = "No codelaw binary: install codelaw in the project (npm or Python) or set codelaw.path";
    status.show();
    log("error", String(status.tooltip));
    return;
  }
  log("info", `using ${binary.path} (${binary.source})`);
  const bin = binary;
  const cwd = folders[0];
  const server: ServerOptions = () => {
    const child = spawn(bin.path, ["lsp", "--stdio"], { cwd, shell: bin.path.endsWith(".cmd") });
    child.stderr?.on("data", (d: Buffer) => {
      for (const line of d.toString().split(/\r?\n/).filter(Boolean)) {
        stderrTail.push(line);
        log("warn", `server stderr: ${line}`);
      }
    });
    child.on("error", (e) => log("error", `could not start ${bin.path}: ${e.message}`));
    child.on("exit", (code, signal) => {
      const tail = stderrTail.all;
      log(code === 0 ? "info" : "error", `server exited: code ${code ?? "none"}, signal ${signal ?? "none"}${tail.length ? `; last stderr: ${tail.join(" | ")}` : ""}`);
    });
    log("debug", `spawned pid ${child.pid ?? "?"}: ${bin.path} lsp --stdio (cwd ${cwd ?? "none"})`);
    return Promise.resolve(child);
  };
  const options: LanguageClientOptions = {
    documentSelector: [...LANGUAGES.map((language) => ({ scheme: "file", language })), { scheme: "file", pattern: TEXT_CONFIG }],
    outputChannel: clientChannel(),
    initializationOptions: { logLevel: level },
    middleware: {
      // Keep each finding as published; a finding codelaw fixes on save,
      // or one the baseline holds, loses its own squiggle for the
      // extension's green or dotted grey line (decision 8).
      handleDiagnostics: (uri, diagnostics, next) => {
        store.set(uri, diagnostics);
        next(uri, restyle(diagnostics));
      },
      provideHover: async (doc, pos, token, next) => {
        const h = await next(doc, pos, token);
        return h ? decorateHover(store, doc, pos, h) : h;
      },
      // The extension draws the server's badges itself, coloured.
      provideInlayHints: () => [],
      // A manifest's summary and bulk fixes: only while manifests show badges.
      provideCodeLenses: (doc, token, next) => (settings().views.manifests === "badges" ? next(doc, token) : []),
    },
    synchronize: {
      // `codelaw.logLevel` changes reach the server without a restart.
      configurationSection: "codelaw",
      // Config, rules and baseline: the server reloads (SR-EDITOR decision 1).
      fileEvents: vscode.workspace.createFileSystemWatcher("**/{codelaw.toml,pyproject.toml,package.json,.codelaw/**}"),
    },
  };
  client = new LanguageClient("codelaw", "codelaw", server, options);
  client.onDidChangeState((e) => log("debug", `client state ${State[e.oldState]} -> ${State[e.newState]}`));
  try {
    await client.start();
  } catch (e) {
    log("error", `the server failed to start: ${e instanceof Error ? e.message : String(e)}`);
    status.text = "$(warning) codelaw";
    status.tooltip = "The codelaw server failed to start: see codelaw: Show output";
    status.show();
    return;
  }
  const info = client.initializeResult?.serverInfo;
  serverVersion = info?.version;
  log("info", `server ${info?.name ?? "codelaw"} ${info?.version ?? "(no version)"} is running`);
  status.text = "codelaw";
  status.tooltip = `${binary.path} (${binary.source})`;
  status.show();
  // The workspace's manifests are always checked, so the sidebar lists
  // their dependencies before one is opened.
  void openManifests();
}

/** "latest" lines of a manifest: the server's inlay hints of kind `clean`. */
async function cleanLines(doc: vscode.TextDocument): Promise<number[]> {
  if (!client?.initializeResult?.capabilities.inlayHintProvider) return [];
  const end = doc.lineAt(Math.max(doc.lineCount - 1, 0)).range.end;
  const hints = (await client.sendRequest("textDocument/inlayHint", {
    textDocument: { uri: doc.uri.toString() },
    range: { start: { line: 0, character: 0 }, end: { line: end.line, character: end.character } },
  })) as { position: { line: number }; data?: { kind?: string } }[] | null;
  return (hints ?? []).filter((h) => h.data?.kind === "clean").map((h) => h.position.line);
}

function locateLookup(context: vscode.ExtensionContext) {
  return {
    configured: config().get<string | null>("path"),
    folders: (vscode.workspace.workspaceFolders ?? []).map((f) => f.uri.fsPath),
    python: vscode.workspace.getConfiguration("python").get<string>("defaultInterpreterPath"),
    extensionDir: context.extensionPath,
    platform: process.platform,
    exists: (f: string) => fs.existsSync(f),
  };
}

async function stop(): Promise<void> {
  const c = client;
  client = undefined;
  if (c) await c.stop();
}

/** The server's "fix all" edit for a document, or none. */
async function fixAllEdits(doc: vscode.TextDocument): Promise<vscode.TextEdit[]> {
  const whole = new vscode.Range(0, 0, doc.lineCount, 0);
  const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>("vscode.executeCodeActionProvider", doc.uri, whole, FIX_ALL);
  const action = (actions ?? []).find((a) => a.kind?.value === FIX_ALL && a.edit);
  return action?.edit?.get(doc.uri) ?? [];
}

/** codelaw's formatting edits, only when the server offers formatting. */
async function formatEdits(doc: vscode.TextDocument): Promise<vscode.TextEdit[]> {
  if (!client?.initializeResult?.capabilities.documentFormattingProvider) return [];
  const editor = vscode.window.visibleTextEditors.find((e) => e.document === doc);
  const options = { tabSize: Number(editor?.options.tabSize ?? 2), insertSpaces: Boolean(editor?.options.insertSpaces ?? true) };
  const result = await client.sendRequest("textDocument/formatting", { textDocument: { uri: doc.uri.toString() }, options });
  return (await client.protocol2CodeConverter.asTextEdits(result as never)) ?? [];
}

async function applyToActive(name: string, edits: (doc: vscode.TextDocument) => Promise<vscode.TextEdit[]>, none: string): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return;
  const list = await edits(editor.document);
  if (list.length === 0) {
    log("debug", `${name}: the server returned no edits for ${editor.document.uri.fsPath}`);
    vscode.window.showInformationMessage(none);
    return;
  }
  const we = new vscode.WorkspaceEdit();
  we.set(editor.document.uri, list);
  await vscode.workspace.applyEdit(we);
}

/** The rule under the cursor, or one typed in. */
async function ruleToExplain(arg?: string | CardArgs): Promise<string | undefined> {
  if (typeof arg === "string" && arg) return arg;
  if (arg && typeof arg === "object" && arg.rule) return arg.rule;
  const editor = vscode.window.activeTextEditor;
  if (editor) {
    const at = editor.selection.active;
    const d = vscode.languages
      .getDiagnostics(editor.document.uri)
      .find((x) => x.source === "codelaw" && x.range.contains(at));
    if (d?.code) return String(typeof d.code === "object" ? d.code.value : d.code);
  }
  return vscode.window.showInputBox({ prompt: "Rule id", placeHolder: "e.g. js/no-var" });
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  output = vscode.window.createOutputChannel("codelaw", { log: true });
  level = parseLevel(config().get("logLevel"));
  status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right);
  status.command = "codelaw.showOutput";
  context.subscriptions.push(
    output,
    status,
    vscode.commands.registerCommand("codelaw.fixAll", () => applyToActive("fix all", fixAllEdits, "codelaw: nothing to fix")),
    vscode.commands.registerCommand("codelaw.format", () =>
      applyToActive("format", formatEdits, client?.initializeResult?.capabilities.documentFormattingProvider ? "codelaw: already formatted" : "codelaw: this codelaw does not format yet"),
    ),
    vscode.commands.registerCommand("codelaw.explainRule", async (arg?: string | CardArgs) => {
      const id = await ruleToExplain(arg);
      if (!id) return;
      if (!binary) {
        log("error", `explain ${id}: there is no codelaw binary (see the candidates above)`);
        void vscode.window.showWarningMessage("codelaw: no binary found; see codelaw: Show output");
        return;
      }
      const cwd = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
      execFile(binary.path, ["explain", id], { cwd, shell: binary.path.endsWith(".cmd") }, (err, stdout, stderr) => {
        if (err) log("error", `explain ${id} failed: ${err.message}${stderr ? `; stderr: ${stderr.trim()}` : ""}`);
        output.info(stdout || stderr || String(err));
        output.show(true);
      });
    }),
    vscode.commands.registerCommand("codelaw.restartServer", async () => {
      await stop();
      await start(context);
    }),
    vscode.commands.registerCommand("codelaw.showOutput", () => output.show()),
    vscode.commands.registerCommand("codelaw.copyDebugInfo", async () => {
      const settings: Record<string, unknown> = {};
      for (const k of ["enable", "path", "fixOnSave", "formatOnSave", "logLevel", "trace.server", "view.code", "view.manifests", "view.markers", "sidebar", "badges.clean"]) settings[k] = config().get(k);
      const text = renderDebugInfo({
        extension: String(context.extension.packageJSON.version),
        vscode: vscode.version,
        os: `${process.platform} ${process.arch}`,
        server: serverVersion,
        binary,
        tried,
        folders: (vscode.workspace.workspaceFolders ?? []).map((f) => f.uri.fsPath),
        settings,
        log: recent.all,
      });
      await vscode.env.clipboard.writeText(text);
      log("info", "debug info copied to the clipboard");
      void vscode.window.showInformationMessage("codelaw: debug info copied to the clipboard");
    }),
    vscode.workspace.onWillSaveTextDocument((e) => {
      if (!client || e.reason !== vscode.TextDocumentSaveReason.Manual) return;
      const c = config();
      if (c.get<boolean>("fixOnSave")) e.waitUntil(fixAllEdits(e.document));
      else if (c.get<boolean>("formatOnSave")) e.waitUntil(formatEdits(e.document));
    }),
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (e.affectsConfiguration("codelaw.logLevel")) {
        level = parseLevel(config().get("logLevel"));
        log("info", `log level is now ${level}`);
      }
      if (e.affectsConfiguration("codelaw.enable") || e.affectsConfiguration("codelaw.path")) {
        await stop();
        await start(context);
      }
    }),
  );
  activateLook(context);
  await start(context);
}

/** The look of W-137: decorations, the sidebar, the counts, the tabs, the card's actions. */
function activateLook(context: vscode.ExtensionContext): void {
  painter = new Painter(store, cleanLines);
  const sidebar = new Sidebar(store);
  const view = vscode.window.createTreeView("codelaw.findings", { treeDataProvider: sidebar, showCollapseAll: true });
  sidebar.view = view;
  const counts = new Counts(store);
  const tabs = new Tabs(store);
  const p = painter;
  registerActions(context, () => binary?.path);
  context.subscriptions.push(
    store,
    p,
    view,
    counts,
    vscode.window.registerFileDecorationProvider(tabs),
    store.onDidChange((uris) => {
      p.paintUris(uris);
      sidebar.refresh();
      counts.update();
      tabs.changed(uris);
    }),
    vscode.window.onDidChangeVisibleTextEditors(() => p.paintAll()),
    vscode.workspace.onDidChangeTextDocument((e) => {
      // A manifest's clean lines move with its text; the server's next
      // publish repaints the rest.
      if (e.contentChanges.length) p.forget(e.document.uri);
    }),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (["codelaw.view", "codelaw.badges", "codelaw.fixOnSave", "codelaw.sidebar"].some((k) => e.affectsConfiguration(k))) {
        p.paintAll();
        sidebar.refresh();
      }
    }),
  );
}

export function deactivate(): Promise<void> {
  return stop();
}
