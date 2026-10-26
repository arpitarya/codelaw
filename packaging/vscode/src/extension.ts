// The thin VS Code extension (SR-EDITOR decision 4): it finds a codelaw
// binary, starts `codelaw lsp`, and adds commands and a status-bar item.
// It holds no lint or format logic; every verdict comes from the server.

import * as fs from "node:fs";
import { execFile } from "node:child_process";
import * as vscode from "vscode";
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from "vscode-languageclient/node";
import { findBinary, Found } from "./binary";

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

let client: LanguageClient | undefined;
let binary: Found | undefined;
let output: vscode.OutputChannel;
let status: vscode.StatusBarItem;

function config() {
  return vscode.workspace.getConfiguration("codelaw");
}

function locate(context: vscode.ExtensionContext): Found | undefined {
  return findBinary({
    configured: config().get<string | null>("path"),
    folders: (vscode.workspace.workspaceFolders ?? []).map((f) => f.uri.fsPath),
    python: vscode.workspace.getConfiguration("python").get<string>("defaultInterpreterPath"),
    extensionDir: context.extensionPath,
    platform: process.platform,
    exists: (f) => fs.existsSync(f),
  });
}

async function start(context: vscode.ExtensionContext): Promise<void> {
  if (!config().get<boolean>("enable", true)) {
    status.text = "codelaw: off";
    status.show();
    return;
  }
  binary = locate(context);
  if (!binary) {
    status.text = "$(warning) codelaw";
    status.tooltip = "No codelaw binary: install codelaw in the project (npm or Python) or set codelaw.path";
    status.show();
    output.appendLine(String(status.tooltip));
    return;
  }
  output.appendLine(`codelaw: ${binary.path} (${binary.source})`);
  // `--stdio` is appended by the stdio transport.
  const server: ServerOptions = { command: binary.path, args: ["lsp"], transport: TransportKind.stdio, options: { shell: binary.path.endsWith(".cmd") } };
  const options: LanguageClientOptions = {
    documentSelector: LANGUAGES.map((language) => ({ scheme: "file", language })),
    outputChannel: output,
    synchronize: {
      // Config, rules and baseline: the server reloads (SR-EDITOR decision 1).
      fileEvents: vscode.workspace.createFileSystemWatcher("**/{codelaw.toml,pyproject.toml,package.json,.codelaw/**}"),
    },
  };
  client = new LanguageClient("codelaw", "codelaw", server, options);
  await client.start();
  status.text = "codelaw";
  status.tooltip = `${binary.path} (${binary.source})`;
  status.show();
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

async function applyToActive(edits: (doc: vscode.TextDocument) => Promise<vscode.TextEdit[]>, none: string): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return;
  const list = await edits(editor.document);
  if (list.length === 0) {
    vscode.window.showInformationMessage(none);
    return;
  }
  const we = new vscode.WorkspaceEdit();
  we.set(editor.document.uri, list);
  await vscode.workspace.applyEdit(we);
}

/** The rule under the cursor, or one typed in. */
async function ruleToExplain(): Promise<string | undefined> {
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
  output = vscode.window.createOutputChannel("codelaw");
  status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right);
  status.command = "codelaw.showOutput";
  context.subscriptions.push(
    output,
    status,
    vscode.commands.registerCommand("codelaw.fixAll", () => applyToActive(fixAllEdits, "codelaw: nothing to fix")),
    vscode.commands.registerCommand("codelaw.format", () =>
      applyToActive(formatEdits, client?.initializeResult?.capabilities.documentFormattingProvider ? "codelaw: already formatted" : "codelaw: this codelaw does not format yet"),
    ),
    vscode.commands.registerCommand("codelaw.explainRule", async () => {
      const id = await ruleToExplain();
      if (!id || !binary) return;
      const cwd = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
      execFile(binary.path, ["explain", id], { cwd, shell: binary.path.endsWith(".cmd") }, (err, stdout, stderr) => {
        output.appendLine(stdout || stderr || String(err));
        output.show(true);
      });
    }),
    vscode.commands.registerCommand("codelaw.restartServer", async () => {
      await stop();
      await start(context);
    }),
    vscode.commands.registerCommand("codelaw.showOutput", () => output.show()),
    vscode.workspace.onWillSaveTextDocument((e) => {
      if (!client || e.reason !== vscode.TextDocumentSaveReason.Manual) return;
      const c = config();
      if (c.get<boolean>("fixOnSave")) e.waitUntil(fixAllEdits(e.document));
      else if (c.get<boolean>("formatOnSave")) e.waitUntil(formatEdits(e.document));
    }),
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (e.affectsConfiguration("codelaw.enable") || e.affectsConfiguration("codelaw.path")) {
        await stop();
        await start(context);
      }
    }),
  );
  await start(context);
}

export function deactivate(): Promise<void> {
  return stop();
}
