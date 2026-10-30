// The hover card's actions (W-137, board K). Each takes the card's
// `CardArgs`. They edit only through `WorkspaceEdit`s and the editor; the
// one that reaches the network, Refresh, runs `codelaw deps refresh` in a
// terminal the person sees (L5: outside the engine).

import * as fs from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import { CardArgs, commentLeader, isManifest, markerOf } from "./findings";

function rangeOf(a: CardArgs): vscode.Range {
  return new vscode.Range(a.line, a.character, a.endLine, a.endCharacter);
}

async function show(a: CardArgs, selection?: vscode.Range): Promise<vscode.TextEditor> {
  const doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(a.uri));
  return vscode.window.showTextDocument(doc, { selection: selection ?? rangeOf(a) });
}

/** The first line codelaw inserts above `line`, indented as it is. */
async function insertAbove(a: CardArgs, text: (open: string, close: string) => string): Promise<boolean> {
  const doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(a.uri));
  const leader = commentLeader(doc.languageId);
  if (!leader) return false;
  const indent = /^\s*/.exec(doc.lineAt(a.line).text)?.[0] ?? "";
  const edit = new vscode.WorkspaceEdit();
  edit.insert(doc.uri, new vscode.Position(a.line, 0), `${indent}${text(leader.open, leader.close)}\n`);
  return vscode.workspace.applyEdit(edit);
}

function isoIn(days: number): string {
  const d = new Date(Date.now() + days * 86_400_000);
  return d.toISOString().slice(0, 10);
}

function folderOf(uri: string): string | undefined {
  return vscode.workspace.getWorkspaceFolder(vscode.Uri.parse(uri))?.uri.fsPath ?? vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}

async function openAndFind(file: string, needle: string | undefined, message?: string): Promise<void> {
  const doc = await vscode.workspace.openTextDocument(file);
  const at = needle ? doc.getText().indexOf(needle) : -1;
  const pos = at >= 0 ? doc.positionAt(at) : new vscode.Position(0, 0);
  await vscode.window.showTextDocument(doc, { selection: new vscode.Range(pos, pos) });
  if (message) void vscode.window.showInformationMessage(message);
}

export function registerActions(context: vscode.ExtensionContext, binaryPath: () => string | undefined): void {
  const reg = (id: string, f: (a: CardArgs) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(id, f));

  reg("codelaw.applyFix", async (a) => {
    const uri = vscode.Uri.parse(a.uri);
    const actions = (await vscode.commands.executeCommand<vscode.CodeAction[]>("vscode.executeCodeActionProvider", uri, rangeOf(a), vscode.CodeActionKind.QuickFix.value)) ?? [];
    const fix = actions.find((x) => x.title.startsWith(`Fix ${a.rule}`) && x.edit);
    if (fix?.edit) await vscode.workspace.applyEdit(fix.edit);
    else await vscode.commands.executeCommand("codelaw.quickFix", a);
  });

  reg("codelaw.quickFix", async (a) => {
    await show(a);
    await vscode.commands.executeCommand("editor.action.quickFix");
  });

  reg("codelaw.askAgent", async (a) => {
    const file = vscode.workspace.asRelativePath(vscode.Uri.parse(a.uri));
    const prompt = `Fix the codelaw finding ${a.rule} at ${file}:${a.line + 1}: ${a.message}. \`codelaw explain ${a.rule}\` says what the rule wants; run \`codelaw check ${file}\` when done.`;
    try {
      await vscode.commands.executeCommand("workbench.action.chat.open", { query: prompt });
    } catch {
      await vscode.env.clipboard.writeText(prompt);
      void vscode.window.showInformationMessage("codelaw: the request for your agent is on the clipboard");
    }
  });

  reg("codelaw.ignoreWithExpiry", async (a) => {
    const expires = await vscode.window.showInputBox({ prompt: `Ignore ${a.rule} until (the first day it no longer holds)`, value: isoIn(30), validateInput: (v) => (/^\d{4}-\d{2}-\d{2}$/.test(v) ? undefined : "YYYY-MM-DD") });
    if (!expires) return;
    const reason = await vscode.window.showInputBox({ prompt: "Why", placeHolder: "the reason a reviewer reads" });
    if (!reason) return;
    const fsPath = vscode.Uri.parse(a.uri).fsPath;
    if (isManifest(fsPath)) {
      // A manifest has no comments: the dependency's exception lives in `[deps] ignore`.
      const eco = path.basename(fsPath) === "pyproject.toml" ? "pypi" : "npm";
      const name = a.message.match(/`([^`\s]+)`/)?.[1] ?? "<package>";
      const snippet = `[deps]\nignore = [{ package = "${eco}:${name}", expires = "${expires}" }]  # ${reason}\n`;
      await vscode.env.clipboard.writeText(snippet);
      const root = folderOf(a.uri);
      const toml = root ? path.join(root, "codelaw.toml") : undefined;
      if (toml && fs.existsSync(toml)) await openAndFind(toml, "[deps]", "codelaw: the `[deps] ignore` entry is on the clipboard — add it to codelaw.toml");
      else void vscode.window.showInformationMessage("codelaw: the `[deps] ignore` entry is on the clipboard — add it to codelaw.toml");
      return;
    }
    const done = await insertAbove(a, (open, close) => `${open}codelaw-ignore ${a.rule} expires=${expires}: ${reason}${close}`);
    if (!done) void vscode.window.showWarningMessage("codelaw: this file has no comments to hold an ignore");
  });

  reg("codelaw.changeDate", async (a) => {
    const doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(a.uri));
    const text = doc.lineAt(a.line).text;
    const m = /\d{4}-\d{2}-\d{2}/.exec(text);
    await show(a, m ? new vscode.Range(a.line, m.index, a.line, m.index + 10) : undefined);
  });

  reg("codelaw.addTicket", async (a) => {
    const ticket = await vscode.window.showInputBox({ prompt: "Ticket", placeHolder: "ABC-123" });
    if (!ticket) return;
    const doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(a.uri));
    const text = doc.lineAt(a.line).text;
    const m = markerOf(text);
    if (!m) return;
    const edit = new vscode.WorkspaceEdit();
    const after = m.tag[1];
    if (text[after] === "(") edit.insert(doc.uri, new vscode.Position(a.line, after + 1), `${ticket}, `);
    else edit.insert(doc.uri, new vscode.Position(a.line, after), `(${ticket})`);
    await vscode.workspace.applyEdit(edit);
  });

  reg("codelaw.addSkipReason", async (a) => {
    const ticket = await vscode.window.showInputBox({ prompt: "Ticket", placeHolder: "ABC-123" });
    if (!ticket) return;
    const until = await vscode.window.showInputBox({ prompt: "Skipped until", value: isoIn(30), validateInput: (v) => (/^\d{4}-\d{2}-\d{2}$/.test(v) ? undefined : "YYYY-MM-DD") });
    if (!until) return;
    const why = await vscode.window.showInputBox({ prompt: "Why it is off" });
    if (!why) return;
    await insertAbove(a, (open, close) => `${open}SKIP(${ticket}, ${until}): ${why}${close}`);
  });

  reg("codelaw.openManifest", async (a) => {
    let dir = path.dirname(vscode.Uri.parse(a.uri).fsPath);
    const root = folderOf(a.uri);
    const pkg = a.message.match(/`([^`\s]+)`/)?.[1];
    for (;;) {
      for (const name of ["package.json", "pyproject.toml"]) {
        const f = path.join(dir, name);
        if (fs.existsSync(f) && (!pkg || fs.readFileSync(f, "utf8").includes(pkg))) return openAndFind(f, pkg ? `"${pkg}` : undefined);
      }
      const up = path.dirname(dir);
      if (up === dir || (root && !up.startsWith(root))) break;
      dir = up;
    }
    void vscode.window.showWarningMessage("codelaw: no package.json or pyproject.toml above this file names it");
  });

  context.subscriptions.push(
    vscode.commands.registerCommand("codelaw.stopFixOnSave", () => vscode.workspace.getConfiguration("codelaw").update("fixOnSave", false, vscode.ConfigurationTarget.Workspace)),
    vscode.commands.registerCommand("codelaw.openFlags", async (a?: CardArgs) => {
      const root = a ? folderOf(a.uri) : vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
      const flags = root ? path.join(root, ".codelaw", "flags.toml") : undefined;
      if (flags && fs.existsSync(flags)) await openAndFind(flags, undefined);
      else void vscode.window.showInformationMessage("codelaw: this project has no .codelaw/flags.toml");
    }),
    vscode.commands.registerCommand("codelaw.refreshDeps", () => {
      const bin = binaryPath();
      if (!bin) {
        void vscode.window.showWarningMessage("codelaw: no binary found; see codelaw: Show output");
        return;
      }
      const term = vscode.window.createTerminal({ name: "codelaw deps refresh", cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath });
      term.show();
      term.sendText(`"${bin}" deps refresh`);
    }),
    vscode.commands.registerCommand("codelaw.turnSourceOff", async (source: string) => {
      const root = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
      const toml = root ? path.join(root, "codelaw.toml") : undefined;
      const say = `codelaw: to stop reading ${source}, set \`enabled = false\` on its entry of \`[deps] advisory_sources\` in codelaw.toml`;
      if (toml && fs.existsSync(toml)) await openAndFind(toml, source, say);
      else void vscode.window.showInformationMessage(say);
    }),
  );
}
