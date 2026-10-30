// The codelaw sidebar (W-137, board D): Dependencies, Deadlines, Tests and
// Ownership, each group a kind of finding, every row a finding to open;
// above them, while the dependency scan is incomplete, its banner with
// Refresh and "turn the source off". Built from what the server has
// published: the open files, plus each workspace folder's manifests, which
// the extension opens (without showing them) so their dependencies are
// always listed.

import * as vscode from "vscode";
import { Store } from "./decorations";
import { countsOf, Finding, iconOf, missingSources, rowOf, SECTIONS, toneOf } from "./findings";

type Node =
  | { type: "banner"; message: string }
  | { type: "action"; label: string; command: vscode.Command; icon: string }
  | { type: "section"; index: number; count: number }
  | { type: "group"; label: string; findings: Finding[] }
  | { type: "finding"; finding: Finding };

const TONE_COLOUR: Record<string, string> = {
  error: "codelaw.badgeErrorForeground",
  warn: "codelaw.badgeWarningForeground",
  info: "codelaw.badgeInfoForeground",
  ok: "codelaw.badgeOkForeground",
  muted: "codelaw.badgeMutedForeground",
};

export class Sidebar implements vscode.TreeDataProvider<Node> {
  private readonly emitter = new vscode.EventEmitter<Node | undefined>();
  readonly onDidChangeTreeData = this.emitter.event;
  view: vscode.TreeView<Node> | undefined;

  constructor(private readonly store: Store) {}

  refresh(): void {
    this.emitter.fire(undefined);
    const scan = countsOf(this.store.all()).scan;
    if (this.view) this.view.message = scan ? `⚠ Scan incomplete — ${scan}` : undefined;
  }

  private ofKinds(kinds: string[]): Finding[] {
    return this.store.all().filter((f) => f.data.kind !== undefined && kinds.includes(f.data.kind));
  }

  getChildren(node?: Node): Node[] {
    if (!node) {
      const out: Node[] = [];
      const scan = countsOf(this.store.all()).scan;
      if (scan) out.push({ type: "banner", message: scan });
      SECTIONS.forEach((s, index) => {
        const count = s.groups.reduce((n, g) => n + this.ofKinds(g.kinds).length, 0);
        out.push({ type: "section", index, count });
      });
      return out;
    }
    switch (node.type) {
      case "banner": {
        const actions: Node[] = [{ type: "action", label: "Refresh", icon: "refresh", command: { title: "Refresh", command: "codelaw.refreshDeps" } }];
        for (const source of missingSources(node.message)) {
          actions.push({ type: "action", label: `Turn ${source} off`, icon: "circle-slash", command: { title: "Turn off", command: "codelaw.turnSourceOff", arguments: [source] } });
        }
        return actions;
      }
      case "section":
        return SECTIONS[node.index].groups
          .map((g) => ({ type: "group" as const, label: g.label, findings: this.ofKinds(g.kinds) }))
          .filter((g) => g.findings.length > 0);
      case "group":
        return [...node.findings].sort((a, b) => a.severity - b.severity || a.uri.localeCompare(b.uri) || a.line - b.line).map((finding) => ({ type: "finding" as const, finding }));
      default:
        return [];
    }
  }

  getTreeItem(node: Node): vscode.TreeItem {
    switch (node.type) {
      case "banner": {
        const item = new vscode.TreeItem("Scan incomplete", vscode.TreeItemCollapsibleState.Expanded);
        item.description = node.message;
        item.tooltip = node.message;
        item.iconPath = new vscode.ThemeIcon("warning", new vscode.ThemeColor("codelaw.badgeWarningForeground"));
        return item;
      }
      case "action": {
        const item = new vscode.TreeItem(node.label, vscode.TreeItemCollapsibleState.None);
        item.command = node.command;
        item.iconPath = new vscode.ThemeIcon(node.icon);
        return item;
      }
      case "section": {
        const item = new vscode.TreeItem(SECTIONS[node.index].label.toUpperCase(), node.count ? vscode.TreeItemCollapsibleState.Expanded : vscode.TreeItemCollapsibleState.None);
        item.description = String(node.count);
        return item;
      }
      case "group": {
        const worst = Math.min(...node.findings.map((f) => f.severity));
        const first = node.findings.find((f) => f.severity === worst) ?? node.findings[0];
        const item = new vscode.TreeItem(node.label, vscode.TreeItemCollapsibleState.Expanded);
        item.description = String(node.findings.length);
        item.iconPath = new vscode.ThemeIcon("circle-filled", new vscode.ThemeColor(TONE_COLOUR[toneOf(first.data, worst)]));
        return item;
      }
      case "finding": {
        const f = node.finding;
        const row = rowOf(f);
        const item = new vscode.TreeItem(row.label, vscode.TreeItemCollapsibleState.None);
        item.description = [row.description, row.grade].filter(Boolean).join("  ");
        item.tooltip = f.message;
        item.iconPath = new vscode.ThemeIcon(iconOf(f.data, f.severity), new vscode.ThemeColor(TONE_COLOUR[toneOf(f.data, f.severity)]));
        item.command = {
          title: "Open",
          command: "vscode.open",
          arguments: [vscode.Uri.parse(f.uri), { selection: new vscode.Range(f.line, f.character, f.endLine, f.endCharacter) }],
        };
        return item;
      }
    }
  }
}

/** Open (not show) each workspace folder's manifests, so the server checks them. */
export async function openManifests(): Promise<void> {
  for (const folder of vscode.workspace.workspaceFolders ?? []) {
    for (const name of ["package.json", "pyproject.toml"]) {
      const uri = vscode.Uri.joinPath(folder.uri, name);
      try {
        await vscode.workspace.fs.stat(uri);
        await vscode.workspace.openTextDocument(uri);
      } catch {
        // No such manifest.
      }
    }
  }
}
