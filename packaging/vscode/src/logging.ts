// What the extension logs (SR-EDITOR decision 7). No vscode import: this is
// tested under plain Node. Paths, rule ids, counts and timings only; never
// source text, never environment variables.

export type Level = "error" | "warn" | "info" | "debug";

export const LEVELS: Level[] = ["error", "warn", "info", "debug"];

/** `codelaw.logLevel`, or `info` for anything else. */
export function parseLevel(v: unknown): Level {
  return LEVELS.includes(v as Level) ? (v as Level) : "info";
}

/** The last `max` lines, for *Copy debug info*. */
export class Ring {
  private lines: string[] = [];
  constructor(private readonly max = 200) {}
  push(line: string): void {
    this.lines.push(line);
    if (this.lines.length > this.max) this.lines.splice(0, this.lines.length - this.max);
  }
  get all(): string[] {
    return [...this.lines];
  }
}

/**
 * The language client writes `[Error - 10:01:02] text`, and the server's
 * debug lines (`LOG`) arrive bare. The log channel timestamps and levels
 * lines itself, so the client's prefix is parsed off and used as the level.
 */
export function fromClientLine(line: string): { level: Level; text: string } {
  const m = /^\[(Error|Warn|Info|Debug)\s*- [^\]]*\] ([\s\S]*)$/.exec(line);
  if (!m) return { level: "debug", text: line };
  return { level: m[1].toLowerCase() as Level, text: m[2] };
}

export interface DebugInfo {
  extension: string;
  vscode: string;
  os: string;
  server?: string;
  binary?: { path: string; source: string };
  tried: { source: string; path: string; verdict: string }[];
  folders: string[];
  settings: Record<string, unknown>;
  log: string[];
}

/** The text *Copy debug info* puts on the clipboard. */
export function renderDebugInfo(d: DebugInfo): string {
  const out = [
    "codelaw debug info",
    `extension: ${d.extension}`,
    `vscode: ${d.vscode}`,
    `os: ${d.os}`,
    `server: ${d.server ?? "not running"}`,
    `binary: ${d.binary ? `${d.binary.path} (${d.binary.source})` : "none found"}`,
    "candidates:",
    ...d.tried.map((t) => `  ${t.source}: ${t.path} - ${t.verdict}`),
    `workspace folders: ${d.folders.join(", ") || "none"}`,
    "settings:",
    ...Object.entries(d.settings).map(([k, v]) => `  codelaw.${k}: ${JSON.stringify(v)}`),
    `last ${d.log.length} log lines:`,
    ...d.log,
  ];
  return out.join("\n");
}
