// The status bar's counts and the tabs' colours (W-137, board D):
// `$(shield) 3 vulnerable`, `$(arrow-up) 3 outdated`, `$(clock) 2 expired ·
// 2 due`, a warning item while the dependency scan is incomplete, and a
// FileDecorationProvider that colours a file's name and gives its count.

import * as vscode from "vscode";
import { Store } from "./decorations";
import { countsOf, fileBadge } from "./findings";

export class Counts implements vscode.Disposable {
  private readonly vulnerable = vscode.window.createStatusBarItem("codelaw.vulnerable", vscode.StatusBarAlignment.Right, 101);
  private readonly outdated = vscode.window.createStatusBarItem("codelaw.outdated", vscode.StatusBarAlignment.Right, 101);
  private readonly deadlines = vscode.window.createStatusBarItem("codelaw.deadlines", vscode.StatusBarAlignment.Right, 101);
  private readonly scan = vscode.window.createStatusBarItem("codelaw.scan", vscode.StatusBarAlignment.Right, 100);

  constructor(private readonly store: Store) {
    for (const item of [this.vulnerable, this.outdated, this.deadlines, this.scan]) {
      item.name = "codelaw";
      item.command = "codelaw.findings.focus";
    }
    this.scan.backgroundColor = new vscode.ThemeColor("statusBarItem.warningBackground");
  }

  update(): void {
    const c = countsOf(this.store.all());
    show(this.vulnerable, c.vulnerable > 0, `$(shield) ${c.vulnerable} vulnerable`, "codelaw: vulnerable dependencies");
    show(this.outdated, c.outdated > 0, `$(arrow-up) ${c.outdated} outdated`, "codelaw: outdated or deprecated dependencies");
    const words = [c.expired ? `${c.expired} expired` : "", c.due ? `${c.due} due` : ""].filter(Boolean).join(" · ");
    show(this.deadlines, words.length > 0, `$(clock) ${words}`, "codelaw: TODOs and flags past or near their date");
    show(this.scan, c.scan !== undefined, "$(warning) codelaw · scan incomplete", c.scan ?? "");
  }

  dispose(): void {
    for (const item of [this.vulnerable, this.outdated, this.deadlines, this.scan]) item.dispose();
  }
}

function show(item: vscode.StatusBarItem, on: boolean, text: string, tooltip: string): void {
  if (!on) {
    item.hide();
    return;
  }
  item.text = text;
  item.tooltip = tooltip;
  item.show();
}

const FILE_COLOUR = { error: "codelaw.fileError", warn: "codelaw.fileWarning", info: "codelaw.fileInfo", ok: "codelaw.fileInfo", muted: "codelaw.fileInfo" };

/** A file's tab and explorer entry: its count of findings, its worst colour. */
export class Tabs implements vscode.FileDecorationProvider {
  private readonly emitter = new vscode.EventEmitter<vscode.Uri[]>();
  readonly onDidChangeFileDecorations = this.emitter.event;

  constructor(private readonly store: Store) {}

  changed(uris: vscode.Uri[]): void {
    this.emitter.fire(uris);
  }

  provideFileDecoration(uri: vscode.Uri): vscode.FileDecoration | undefined {
    const b = fileBadge(this.store.get(uri));
    if (!b) return undefined;
    return new vscode.FileDecoration(b.badge, `codelaw: ${b.badge} finding(s)`, new vscode.ThemeColor(FILE_COLOUR[b.tone]));
  }
}
