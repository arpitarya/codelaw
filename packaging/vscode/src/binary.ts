// Which codelaw binary the extension starts (SR-EDITOR decision 4;
// SR-PACKAGES decision 6). The project's own codelaw wins, so the editor
// agrees with CI. No vscode import: this is tested under plain Node.

import * as path from "node:path";

export interface Lookup {
  /** `codelaw.path`, if set. A relative path is from the first folder. */
  configured?: string | null;
  /** The workspace folders, first one first. */
  folders: string[];
  /** The selected Python interpreter, if the editor knows one. */
  python?: string | null;
  /** Where the extension is installed; the bundled binary is in `bin/`. */
  extensionDir: string;
  platform: NodeJS.Platform;
  exists: (file: string) => boolean;
}

export type Source = "codelaw.path" | "node_modules" | "venv" | "python" | "bundled";

export interface Found {
  path: string;
  source: Source;
}

/** The binary to run, or `undefined` when there is none anywhere. */
export function findBinary(l: Lookup): Found | undefined {
  const win = l.platform === "win32";
  const p = win ? path.win32 : path.posix;
  const exe = win ? "codelaw.exe" : "codelaw";
  if (l.configured) {
    const base = l.folders[0] ?? l.extensionDir;
    return { path: p.resolve(base, l.configured), source: "codelaw.path" };
  }
  for (const folder of l.folders) {
    const npm = p.join(folder, "node_modules", ".bin", win ? "codelaw.cmd" : "codelaw");
    if (l.exists(npm)) return { path: npm, source: "node_modules" };
    const venv = p.join(folder, ".venv", win ? "Scripts" : "bin", exe);
    if (l.exists(venv)) return { path: venv, source: "venv" };
  }
  if (l.python) {
    const beside = p.join(p.dirname(l.python), exe);
    if (l.exists(beside)) return { path: beside, source: "python" };
  }
  const bundled = p.join(l.extensionDir, "bin", exe);
  if (l.exists(bundled)) return { path: bundled, source: "bundled" };
  return undefined;
}
