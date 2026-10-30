// The findings store and everything drawn in the editor from it (W-137,
// docs/design/editor): squiggle colours, gutter glyphs, row tints,
// end-of-line badges, TODO/ticket/date chips and the hover card's icon and
// actions. Drawn with the VS Code API alone (L11). Every colour is a
// `contributes.colors` entry, except the gutter glyphs: VS Code draws those
// as images, so they carry the Dark+/Light+ defaults of the same colours.

import * as vscode from "vscode";
import {
  actionsOf,
  badgeOf,
  CardArgs,
  CARD_COMMANDS,
  categoryOf,
  DEFAULT_VIEWS,
  Finding,
  FindingData,
  footerOf,
  HINT,
  iconOf,
  markerOf,
  parseView,
  squiggleOf,
  Tone,
  toneOf,
  viewFor,
  ViewSettings,
} from "./findings";

export function settings(): { views: ViewSettings; clean: boolean; fixOnSave: boolean; sidebar: boolean } {
  const c = vscode.workspace.getConfiguration("codelaw");
  return {
    views: {
      code: parseView(c.get("view.code"), DEFAULT_VIEWS.code),
      manifests: parseView(c.get("view.manifests"), DEFAULT_VIEWS.manifests),
      markers: parseView(c.get("view.markers"), DEFAULT_VIEWS.markers),
    },
    clean: c.get<boolean>("badges.clean", true),
    fixOnSave: c.get<boolean>("fixOnSave", false),
    sidebar: c.get<boolean>("sidebar", true),
  };
}

/** Every codelaw finding by document, as the server published it. */
export class Store {
  private readonly by = new Map<string, Finding[]>();
  private readonly emitter = new vscode.EventEmitter<vscode.Uri[]>();
  readonly onDidChange = this.emitter.event;

  set(uri: vscode.Uri, diagnostics: readonly vscode.Diagnostic[]): void {
    const list = diagnostics.filter((d) => d.source === "codelaw").map((d) => toFinding(uri, d));
    if (list.length) this.by.set(uri.toString(), list);
    else this.by.delete(uri.toString());
    this.emitter.fire([uri]);
  }

  get(uri: vscode.Uri | string): Finding[] {
    return this.by.get(uri.toString()) ?? [];
  }

  all(): Finding[] {
    return Array.from(this.by.values()).flat();
  }

  uris(): string[] {
    return Array.from(this.by.keys());
  }

  dispose(): void {
    this.emitter.dispose();
  }
}

function toFinding(uri: vscode.Uri, d: vscode.Diagnostic): Finding {
  const data = ((d as unknown as { data?: FindingData }).data ?? {}) as FindingData;
  const code = typeof d.code === "object" ? d.code.value : d.code;
  return {
    uri: uri.toString(),
    line: d.range.start.line,
    character: d.range.start.character,
    endLine: d.range.end.line,
    endCharacter: d.range.end.character,
    severity: d.severity,
    message: d.message,
    rule: String(code ?? data.rule ?? "codelaw"),
    data,
  };
}

/**
 * What the Problems panel and the squiggle get: a finding codelaw fixes on
 * save, or one the baseline holds, becomes a hint (no squiggle of its
 * own), and the extension draws its green or dotted grey line instead.
 */
export function restyle(diagnostics: vscode.Diagnostic[]): vscode.Diagnostic[] {
  const { fixOnSave } = settings();
  for (const d of diagnostics) {
    if (d.source !== "codelaw") continue;
    const data = (d as unknown as { data?: FindingData }).data ?? {};
    if (squiggleOf(data, fixOnSave) !== "severity") d.severity = vscode.DiagnosticSeverity.Hint;
  }
  return diagnostics;
}

// ---- gutter glyphs ----------------------------------------------------

const GLYPHS: Record<string, string> = {
  shield: '<path fill="C" d="M8 1 14 3v5c0 3.5-3 6-6 7-3-1-6-3.5-6-7V3z"/>',
  "circle-slash": '<circle cx="8" cy="8" r="5.5" fill="none" stroke="C" stroke-width="1.8"/><path d="m4.2 11.8 7.6-7.6" stroke="C" stroke-width="1.8"/>',
  "arrow-up": '<path fill="C" d="M8 2 13.5 7.5H10V14H6V7.5H2.5z"/>',
  check: '<path d="M3 8.5 6.5 12 13 4.5" fill="none" stroke="C" stroke-width="2"/>',
  clock: '<circle cx="8" cy="8" r="5.8" fill="none" stroke="C" stroke-width="1.6"/><path d="M8 4.5V8l2.5 1.6" fill="none" stroke="C" stroke-width="1.6"/>',
  flag: '<path d="M3.5 1.5v13" stroke="C" stroke-width="1.6"/><path fill="C" d="M4 2h8.5l-2 3 2 3H4z"/>',
  target: '<circle cx="8" cy="8" r="6" fill="none" stroke="C" stroke-width="1.5"/><circle cx="8" cy="8" r="2.5" fill="C"/>',
  "debug-pause": '<rect x="4" y="3" width="2.6" height="10" fill="C"/><rect x="9.4" y="3" width="2.6" height="10" fill="C"/>',
  warning: '<path fill="C" d="M8 1.5 15 14.5H1z"/>',
  person: '<circle cx="8" cy="5" r="3" fill="C"/><path fill="C" d="M2.5 15c0-3.3 2.4-5.5 5.5-5.5s5.5 2.2 5.5 5.5z"/>',
  error: '<circle cx="8" cy="8" r="6" fill="C"/>',
  info: '<circle cx="8" cy="8" r="6" fill="C"/>',
  "circle-large-outline": '<circle cx="8" cy="8" r="5.5" fill="none" stroke="C" stroke-width="1.5" stroke-dasharray="2 2"/>',
};

const DARK: Record<Tone, string> = { error: "#f14c4c", warn: "#cca700", info: "#3794ff", ok: "#73c991", muted: "#8a8a8a" };
const LIGHT: Record<Tone, string> = { error: "#e51400", warn: "#bf8803", info: "#1a85ff", ok: "#388a34", muted: "#6e6e6e" };

function glyph(icon: string, colour: string): vscode.Uri {
  const body = (GLYPHS[icon] ?? GLYPHS.info).replace(/"C"/g, `"${colour}"`);
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">${body}</svg>`;
  return vscode.Uri.parse(`data:image/svg+xml;base64,${Buffer.from(svg).toString("base64")}`);
}

// ---- decoration types -------------------------------------------------

const color = (id: string) => new vscode.ThemeColor(id);
const TONE_ID: Record<Tone, string> = { error: "Error", warn: "Warning", info: "Info", ok: "Ok", muted: "Muted" };

export class Painter implements vscode.Disposable {
  private readonly gutters = new Map<string, vscode.TextEditorDecorationType>();
  private readonly badges = new Map<Tone, vscode.TextEditorDecorationType>();
  private readonly tints: Record<"error" | "warn", vscode.TextEditorDecorationType>;
  private readonly fixOnSave: vscode.TextEditorDecorationType;
  private readonly baseline: vscode.TextEditorDecorationType;
  private readonly chips: Record<"tag" | "tagLate" | "ticket" | "date" | "dateLate", vscode.TextEditorDecorationType>;
  /** "latest" lines of each manifest, from the server's inlay hints. */
  private readonly clean = new Map<string, { version: number; lines: number[] }>();

  constructor(
    private readonly store: Store,
    private readonly hints: (doc: vscode.TextDocument) => Promise<number[]>,
  ) {
    for (const tone of Object.keys(TONE_ID) as Tone[]) {
      const id = TONE_ID[tone];
      this.badges.set(
        tone,
        vscode.window.createTextEditorDecorationType({
          after: {
            color: color(`codelaw.badge${id}Foreground`),
            backgroundColor: color(`codelaw.badge${id}Background`),
            border: "1px solid",
            borderColor: color(`codelaw.badge${id}Border`),
            margin: "0 0 0 1.5em",
            fontWeight: "normal",
          },
          rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
        }),
      );
    }
    const tint = (id: string) => vscode.window.createTextEditorDecorationType({ isWholeLine: true, backgroundColor: color(id) });
    this.tints = { error: tint("codelaw.rowErrorBackground"), warn: tint("codelaw.rowWarningBackground") };
    // The editor exposes every theme colour as a CSS variable.
    this.fixOnSave = vscode.window.createTextEditorDecorationType({ textDecoration: "underline wavy var(--vscode-codelaw-squiggleFixOnSave)" });
    this.baseline = vscode.window.createTextEditorDecorationType({ textDecoration: "underline dotted var(--vscode-codelaw-squiggleBaseline)" });
    const chip = (bg: string, fg: string) =>
      vscode.window.createTextEditorDecorationType({ backgroundColor: color(bg), color: color(fg), borderRadius: "3px", fontWeight: "bold" });
    this.chips = {
      tag: chip("codelaw.chipTagBackground", "codelaw.chipTagForeground"),
      tagLate: chip("codelaw.chipLateBackground", "codelaw.chipLateForeground"),
      ticket: chip("codelaw.chipTicketBackground", "codelaw.chipTicketForeground"),
      date: chip("codelaw.chipDateBackground", "codelaw.chipDateForeground"),
      dateLate: chip("codelaw.chipLateBackground", "codelaw.chipLateForeground"),
    };
  }

  private gutter(icon: string, tone: Tone): vscode.TextEditorDecorationType {
    const key = `${icon}/${tone}`;
    let t = this.gutters.get(key);
    if (!t) {
      t = vscode.window.createTextEditorDecorationType({
        dark: { gutterIconPath: glyph(icon, DARK[tone]) },
        light: { gutterIconPath: glyph(icon, LIGHT[tone]) },
        gutterIconSize: "75%",
      });
      this.gutters.set(key, t);
    }
    return t;
  }

  /** Repaint every visible editor (only those: SR-EDITOR decision 8). */
  paintAll(): void {
    for (const e of vscode.window.visibleTextEditors) void this.paint(e);
  }

  /** Repaint the visible editors of some documents. */
  paintUris(uris: readonly vscode.Uri[]): void {
    const keys = new Set(uris.map((u) => u.toString()));
    for (const e of vscode.window.visibleTextEditors) if (keys.has(e.document.uri.toString())) void this.paint(e);
  }

  async paint(editor: vscode.TextEditor): Promise<void> {
    const doc = editor.document;
    const s = settings();
    const findings = this.store.get(doc.uri);
    const path = doc.uri.fsPath;
    const gutter = new Map<vscode.TextEditorDecorationType, vscode.Range[]>();
    const badge = new Map<Tone, vscode.DecorationOptions[]>();
    const tint: Record<"error" | "warn", vscode.Range[]> = { error: [], warn: [] };
    const green: vscode.Range[] = [];
    const grey: vscode.Range[] = [];
    const lines = new Set<number>();
    // Most severe first, so each line's glyph and badge are its worst.
    const ordered = [...findings].sort((a, b) => a.severity - b.severity);
    for (const f of ordered) {
      const range = new vscode.Range(f.line, f.character, f.endLine, f.endCharacter);
      const look = squiggleOf(f.data, s.fixOnSave);
      if (look === "fixOnSave") green.push(range);
      if (look === "baseline") grey.push(range);
      const view = viewFor(categoryOf(f.data, path), s.views);
      if (view !== "badges" || lines.has(f.line) || f.line >= doc.lineCount) continue;
      lines.add(f.line);
      const severity = look === "severity" ? f.severity : HINT;
      const tone = toneOf(f.data, f.severity);
      push(gutter, this.gutter(iconOf(f.data, f.severity), tone), new vscode.Range(f.line, 0, f.line, 0));
      if ((tone === "error" || tone === "warn") && severity !== HINT) tint[tone].push(doc.lineAt(f.line).range);
      const end = doc.lineAt(f.line).range.end;
      push(badge, tone, { range: new vscode.Range(end, end), renderOptions: { after: { contentText: ` ${badgeOf(f)} ` } } });
    }
    // "latest" beside a clean dependency, in a manifest drawn with badges.
    const manifestBadges = s.views.manifests === "badges" && categoryOf({}, path) === "manifest";
    if (manifestBadges && s.clean) {
      for (const line of await this.cleanLines(doc)) {
        if (lines.has(line) || line >= doc.lineCount) continue;
        push(gutter, this.gutter("check", "ok"), new vscode.Range(line, 0, line, 0));
        const end = doc.lineAt(line).range.end;
        push(badge, "ok", { range: new vscode.Range(end, end), renderOptions: { after: { contentText: " latest " } } });
      }
    }
    // Chips in TODO/FIXME/HACK/SKIP comments, when markers are badges.
    const chips = { tag: [] as vscode.Range[], tagLate: [] as vscode.Range[], ticket: [] as vscode.Range[], date: [] as vscode.Range[], dateLate: [] as vscode.Range[] };
    if (s.views.markers === "badges" && categoryOf({}, path) === "code" && doc.lineCount < 20000) {
      const late = new Set(findings.filter((f) => f.data.kind === "expired" || f.data.kind === "ticket").map((f) => f.line));
      for (let i = 0; i < doc.lineCount; i++) {
        const text = doc.lineAt(i).text;
        if (!/TODO|FIXME|HACK|XXX|SKIP/.test(text)) continue;
        const m = markerOf(text);
        if (!m) continue;
        const r = ([a, b]: [number, number]) => new vscode.Range(i, a, i, b);
        (late.has(i) ? chips.tagLate : chips.tag).push(r(m.tag));
        if (m.ticket) chips.ticket.push(r(m.ticket));
        if (m.date) (findings.some((f) => f.line === i && f.data.kind === "expired") ? chips.dateLate : chips.date).push(r(m.date));
      }
    }
    for (const type of this.gutters.values()) editor.setDecorations(type, gutter.get(type) ?? []);
    for (const [tone, type] of this.badges) editor.setDecorations(type, badge.get(tone) ?? []);
    editor.setDecorations(this.tints.error, tint.error);
    editor.setDecorations(this.tints.warn, tint.warn);
    editor.setDecorations(this.fixOnSave, green);
    editor.setDecorations(this.baseline, grey);
    for (const k of Object.keys(chips) as (keyof typeof chips)[]) editor.setDecorations(this.chips[k], chips[k]);
  }

  private async cleanLines(doc: vscode.TextDocument): Promise<number[]> {
    const key = doc.uri.toString();
    const kept = this.clean.get(key);
    if (kept && kept.version === doc.version) return kept.lines;
    const lines = await this.hints(doc).catch(() => []);
    this.clean.set(key, { version: doc.version, lines });
    return lines;
  }

  forget(uri: vscode.Uri): void {
    this.clean.delete(uri.toString());
  }

  dispose(): void {
    for (const t of [...this.gutters.values(), ...this.badges.values(), this.tints.error, this.tints.warn, this.fixOnSave, this.baseline, ...Object.values(this.chips)]) t.dispose();
  }
}

function push<K, V>(map: Map<K, V[]>, key: K, value: V): void {
  const list = map.get(key);
  if (list) list.push(value);
  else map.set(key, [value]);
}

// ---- the hover card ---------------------------------------------------

/**
 * The server's card with the icon before its title and its actions after
 * (board K). Each card names its rule in its meta line; that is how a card
 * finds its finding. The markdown trusts only the card's commands.
 */
export function decorateHover(store: Store, doc: vscode.TextDocument, pos: vscode.Position, hover: vscode.Hover): vscode.Hover {
  const here = store.get(doc.uri).filter((f) => new vscode.Range(f.line, f.character, f.endLine, f.endCharacter).contains(pos));
  if (here.length === 0) return hover;
  const { fixOnSave } = settings();
  const parts: string[] = [];
  for (const c of hover.contents) {
    const value = typeof c === "string" ? c : "value" in c ? c.value : "";
    for (const card of value.split("\n\n---\n\n")) {
      const f = here.find((x) => card.includes(`\`${x.rule}\``));
      if (!f) {
        parts.push(card);
        continue;
      }
      let text = `$(${iconOf(f.data, f.severity)}) ${card}`;
      if (squiggleOf(f.data, fixOnSave) === "fixOnSave") text = text.replace("`one-click fix`", "`fixes itself on save`");
      const args: CardArgs = {
        uri: f.uri,
        line: f.line,
        character: f.character,
        endLine: f.endLine,
        endCharacter: f.endCharacter,
        rule: f.rule,
        message: f.message,
        tag: f.data.tag,
      };
      parts.push(`${text}\n\n${footerOf(actionsOf(f.data, doc.uri.fsPath, fixOnSave), args)}`);
    }
  }
  const md = new vscode.MarkdownString(parts.join("\n\n---\n\n"), true);
  md.isTrusted = { enabledCommands: CARD_COMMANDS };
  return new vscode.Hover(md, hover.range);
}
