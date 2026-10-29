// How a finding looks (W-137, docs/design/editor): pure functions from a
// diagnostic's `data` (the server's shape, crates/codelaw-lsp/src/card.rs)
// to an icon, a tone, a badge, a view, the sidebar group and the hover
// card's actions. No vscode import, so the mapping is unit-tested.
//
// Tolerant by design: a `data` without a `kind` (an older server, a rule not
// built yet) is a plain code finding drawn from its severity.

/** VS Code's DiagnosticSeverity values. */
export const ERROR = 0;
export const WARNING = 1;
export const INFO = 2;
export const HINT = 3;

export type View = "squiggles" | "badges";
export type Category = "code" | "manifest" | "marker";
export type Tone = "error" | "warn" | "info" | "ok" | "muted";

/** The server's `data`; every key optional. */
export interface FindingData {
  fix?: string;
  source?: string;
  rule?: string;
  kind?: string;
  view?: string;
  grade?: string;
  title?: string;
  badge?: string;
  package?: string;
  version?: string;
  safe?: string;
  latest?: string;
  advisories?: string[];
  more?: number;
  due?: string;
  days?: number;
  ticket?: string;
  tag?: string;
  flag?: string;
  owner?: string;
  baseline?: boolean;
  fixable?: boolean;
}

/** One finding as the extension keeps it: before any view changes it. */
export interface Finding {
  uri: string;
  line: number;
  character: number;
  endLine: number;
  endCharacter: number;
  severity: number;
  message: string;
  rule: string;
  data: FindingData;
}

const MANIFESTS = [
  "package.json",
  "package-lock.json",
  "pnpm-lock.yaml",
  "yarn.lock",
  "pyproject.toml",
  "uv.lock",
  "poetry.lock",
  "requirements.txt",
  ".nvmrc",
  ".node-version",
  ".python-version",
  ".tool-versions",
];

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

export function isManifest(path: string): boolean {
  return MANIFESTS.includes(baseName(path));
}

/** The view setting a finding follows: the server's word, else the file's. */
export function categoryOf(data: FindingData, path: string): Category {
  if (data.view === "manifest" || data.view === "marker" || data.view === "code") return data.view;
  return isManifest(path) ? "manifest" : "code";
}

export interface ViewSettings {
  code: View;
  manifests: View;
  markers: View;
}

export const DEFAULT_VIEWS: ViewSettings = { code: "squiggles", manifests: "badges", markers: "badges" };

/** A setting's value, or its default when it is not one of the two. */
export function parseView(value: unknown, fallback: View): View {
  return value === "squiggles" || value === "badges" ? value : fallback;
}

export function viewFor(category: Category, views: ViewSettings): View {
  return category === "manifest" ? views.manifests : category === "marker" ? views.markers : views.code;
}

/** The codicon (hover title, sidebar, status bar) and the gutter glyph of a kind. */
const ICONS: Record<string, string> = {
  vulnerable: "shield",
  deprecated: "circle-slash",
  licence: "circle-slash",
  outdated: "arrow-up",
  runtime: "warning",
  scan: "warning",
  expired: "clock",
  due: "clock",
  ticket: "warning",
  flag: "flag",
  "flag-expired": "flag",
  focused: "target",
  skipped: "debug-pause",
  complexity: "warning",
  ownership: "person",
  clean: "check",
};

export function toneOf(data: FindingData, severity: number): Tone {
  if (data.kind === "clean") return "ok";
  if (data.baseline) return "muted";
  return severity === ERROR ? "error" : severity === WARNING ? "warn" : severity === INFO ? "info" : "muted";
}

export function iconOf(data: FindingData, severity: number): string {
  const icon = data.kind ? ICONS[data.kind] : undefined;
  if (icon) return icon;
  if (data.baseline) return "circle-large-outline";
  return severity === ERROR ? "error" : severity === WARNING ? "warning" : "info";
}

/** The words of a badge; never empty, never colour alone. */
export function badgeOf(f: Pick<Finding, "data" | "rule">): string {
  return f.data.badge || f.rule || "codelaw";
}

/**
 * The squiggle a finding gets: the severity's own, green when codelaw fixes
 * it on save (nothing for the person to do), dotted grey when the baseline
 * holds it.
 */
export type Squiggle = "severity" | "fixOnSave" | "baseline";

export function squiggleOf(data: FindingData, fixOnSave: boolean): Squiggle {
  if (data.baseline) return "baseline";
  if (fixOnSave && data.fix === "auto" && data.fixable) return "fixOnSave";
  return "severity";
}

/** The sidebar's sections and their groups, each group a set of kinds. */
export const SECTIONS: { label: string; groups: { label: string; kinds: string[] }[] }[] = [
  {
    label: "Dependencies",
    groups: [
      { label: "Vulnerable", kinds: ["vulnerable"] },
      { label: "Deprecated", kinds: ["deprecated"] },
      { label: "Licence denied", kinds: ["licence"] },
      { label: "Outdated", kinds: ["outdated"] },
      { label: "Runtime mismatch", kinds: ["runtime"] },
    ],
  },
  {
    label: "Deadlines",
    groups: [
      { label: "Expired", kinds: ["expired", "flag-expired"] },
      { label: "Due soon", kinds: ["due", "flag"] },
      { label: "Missing ticket", kinds: ["ticket"] },
    ],
  },
  {
    label: "Tests",
    groups: [
      { label: "Focused (.only)", kinds: ["focused"] },
      { label: "Skipped, no reason", kinds: ["skipped"] },
    ],
  },
  { label: "Ownership", groups: [{ label: "Unowned", kinds: ["ownership"] }] },
];

function ago(days: number | undefined): string {
  if (days === undefined) return "";
  if (days === 0) return "today";
  return days < 0 ? `${-days} d ago` : `in ${days} d`;
}

/** A sidebar row: what it names, and the quieter words beside it. */
export function rowOf(f: Finding): { label: string; description: string; grade?: string } {
  const d = f.data;
  const where = `${baseName(f.uri)}:${f.line + 1}`;
  switch (d.kind) {
    case "vulnerable":
    case "deprecated":
    case "outdated": {
      const to = d.safe ?? d.latest;
      const label = [d.package ?? f.rule, d.version, to ? `→ ${to}` : undefined].filter(Boolean).join(" ");
      return { label, description: where, grade: d.grade?.toUpperCase() };
    }
    case "expired":
    case "due":
    case "ticket":
      return { label: [d.tag ?? "TODO", d.ticket].filter(Boolean).join(" "), description: [where, ago(d.days)].filter(Boolean).join(" · ") };
    case "flag":
    case "flag-expired":
      return { label: `flag ${d.flag ?? ""}`.trim(), description: [where, ago(d.days), d.owner].filter(Boolean).join(" · ") };
    default:
      return { label: d.title ?? f.message, description: where };
  }
}

/** The status bar's counts. */
export interface Counts {
  vulnerable: number;
  outdated: number;
  expired: number;
  due: number;
  scan: string | undefined;
}

export function countsOf(findings: Finding[]): Counts {
  const c: Counts = { vulnerable: 0, outdated: 0, expired: 0, due: 0, scan: undefined };
  for (const f of findings) {
    const k = f.data.kind;
    if (k === "vulnerable") c.vulnerable++;
    else if (k === "outdated" || k === "deprecated") c.outdated++;
    else if (k === "expired" || k === "flag-expired") c.expired++;
    else if (k === "due" || k === "flag") c.due++;
    else if (k === "scan" && c.scan === undefined) c.scan = f.message;
  }
  return c;
}

/** The advisory sources a scan finding names as unavailable: `osv` in `` `osv` (HTTP 403, on …) ``. */
export function missingSources(message: string): string[] {
  const after = message.split("unavailable")[0] ?? message;
  return Array.from(after.matchAll(/`([a-z][a-z0-9_-]*)`/g), (m) => m[1]).filter((s) => !s.includes("/") && s !== "codelaw");
}

/** A file's tab: its count and the tone of its worst finding. */
export function fileBadge(findings: Finding[]): { badge: string; tone: Tone } | undefined {
  const shown = findings.filter((f) => f.severity !== HINT && !f.data.baseline);
  if (shown.length === 0) return undefined;
  const worst = Math.min(...shown.map((f) => f.severity));
  return { badge: shown.length > 9 ? "9+" : String(shown.length), tone: worst === ERROR ? "error" : worst === WARNING ? "warn" : "info" };
}

/** One action at a card's foot: the first is the primary one. */
export interface Action {
  title: string;
  command: string;
}

/** The hover card's actions for one finding (board K). */
export function actionsOf(data: FindingData, path: string, fixOnSave: boolean): Action[] {
  const rule: Action = { title: "Rule page", command: "codelaw.explainRule" };
  const ignore: Action = { title: "Ignore with expiry…", command: "codelaw.ignoreWithExpiry" };
  const quick: Action = { title: "Quick Fix…", command: "codelaw.quickFix" };
  const apply = (title = "Apply fix"): Action => ({ title, command: "codelaw.applyFix" });
  const ask = (title = "Ask agent to fix"): Action => ({ title, command: "codelaw.askAgent" });
  const manifest = isManifest(path);
  if (data.baseline) return [ask(), rule];
  if (squiggleOf(data, fixOnSave) === "fixOnSave") {
    return [{ title: "Fix now", command: "codelaw.fixAll" }, rule, { title: "Stop fixing on save", command: "codelaw.stopFixOnSave" }];
  }
  switch (data.kind) {
    case "vulnerable":
    case "deprecated":
    case "outdated": {
      const to = data.safe ?? data.latest;
      if (!manifest) return [{ title: to ? `Change to ${to}…` : "Open the manifest", command: "codelaw.openManifest" }, ignore];
      return data.fixable ? [apply(to ? `Change to ${to}` : "Apply fix"), quick, ignore] : [quick, ignore];
    }
    case "licence":
    case "runtime":
      return [quick, rule, ignore];
    case "scan":
      return [{ title: "Refresh", command: "codelaw.refreshDeps" }, rule];
    case "expired":
    case "due":
      return [{ title: "Change date…", command: "codelaw.changeDate" }, ask("Ask agent to do it")];
    case "ticket":
      return [{ title: "Add ticket…", command: "codelaw.addTicket" }, rule];
    case "flag":
      return [data.fixable ? apply() : quick, { title: "Open flags.toml", command: "codelaw.openFlags" }];
    case "flag-expired":
      return [{ title: "Pick a branch…", command: "codelaw.quickFix" }, { title: "Open flags.toml", command: "codelaw.openFlags" }];
    case "focused":
      return [data.fixable ? apply() : ask(), rule];
    case "skipped":
      return [{ title: "Add reason…", command: "codelaw.addSkipReason" }, rule];
    case "complexity":
      return [ask("Ask agent to split"), rule, ignore];
    default:
      return [data.fixable ? apply() : ask(), rule, ignore];
  }
}

/** Every command a card may link to: the markdown trusts these and no other. */
export const CARD_COMMANDS = [
  "codelaw.applyFix",
  "codelaw.quickFix",
  "codelaw.askAgent",
  "codelaw.explainRule",
  "codelaw.ignoreWithExpiry",
  "codelaw.fixAll",
  "codelaw.stopFixOnSave",
  "codelaw.openManifest",
  "codelaw.refreshDeps",
  "codelaw.changeDate",
  "codelaw.addTicket",
  "codelaw.addSkipReason",
  "codelaw.openFlags",
];

/** What every card command receives. */
export interface CardArgs {
  uri: string;
  line: number;
  character: number;
  endLine: number;
  endCharacter: number;
  rule: string;
  message: string;
  tag?: string;
}

/** The markdown foot of a card: the primary action bold, the rest plain. */
export function footerOf(actions: Action[], args: CardArgs): string {
  const link = (a: Action) => `[${a.title}](command:${a.command}?${encodeURIComponent(JSON.stringify([args]))})`;
  return actions.map((a, i) => (i === 0 ? `**${link(a)}**` : link(a))).join(" &nbsp;·&nbsp; ");
}

/** The comment that opens a line in a language, for an inserted line. */
export function commentLeader(languageId: string): { open: string; close: string } | undefined {
  if (["python", "yaml", "toml", "shellscript", "dockerfile", "ignore", "properties"].includes(languageId)) return { open: "# ", close: "" };
  if (["javascript", "javascriptreact", "typescript", "typescriptreact", "jsonc", "scss"].includes(languageId)) return { open: "// ", close: "" };
  if (languageId === "css") return { open: "/* ", close: " */" };
  if (languageId === "html" || languageId === "markdown") return { open: "<!-- ", close: " -->" };
  return undefined;
}

/** A TODO/FIXME/HACK/SKIP marker in a comment line: its tag and the chips inside its parentheses. */
export interface Marker {
  tag: [number, number];
  ticket?: [number, number];
  date?: [number, number];
}

const MARKER = /(\/\/|#|\/\*|<!--|^\s*\*)\s*(TODO|FIXME|HACK|XXX|SKIP)\b(\(([^)]*)\))?/;

export function markerOf(line: string): Marker | undefined {
  const m = MARKER.exec(line);
  if (!m || m.index === undefined) return undefined;
  const tagStart = m.index + m[0].indexOf(m[2], m[1].length);
  const out: Marker = { tag: [tagStart, tagStart + m[2].length] };
  if (m[3] !== undefined && m[4] !== undefined) {
    const inner = tagStart + m[2].length + 1;
    const ticket = /[A-Z][A-Z0-9]+-\d+/.exec(m[4]);
    if (ticket) out.ticket = [inner + ticket.index, inner + ticket.index + ticket[0].length];
    const date = /\d{4}-\d{2}-\d{2}/.exec(m[4]);
    if (date) out.date = [inner + date.index, inner + date.index + date[0].length];
  }
  return out;
}
