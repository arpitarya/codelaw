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

/** One place the extension looked, and what it found there. */
export interface Tried {
  source: Source;
  path: string;
  verdict: "used" | "used (not found on disk)" | "missing" | "skipped: an earlier one was used";
}

export interface Trace {
  found: Found | undefined;
  tried: Tried[];
}

/** The binary to run, or `undefined` when there is none anywhere. */
export function findBinary(l: Lookup): Found | undefined {
  return traceBinary(l).found;
}

/**
 * `findBinary` and every candidate it considered, in order, with why each
 * was or was not used (SR-EDITOR decision 7). `codelaw.path` is used
 * whether or not it exists, as before; the trace says when it does not.
 */
export function traceBinary(l: Lookup): Trace {
  const win = l.platform === "win32";
  const p = win ? path.win32 : path.posix;
  const exe = win ? "codelaw.exe" : "codelaw";
  const tried: Tried[] = [];
  let found: Found | undefined;
  const consider = (source: Source, file: string, force = false): void => {
    if (found) {
      tried.push({ source, path: file, verdict: "skipped: an earlier one was used" });
      return;
    }
    const there = l.exists(file);
    if (there || force) found = { path: file, source };
    tried.push({ source, path: file, verdict: there ? "used" : force ? "used (not found on disk)" : "missing" });
  };
  if (l.configured) {
    const base = l.folders[0] ?? l.extensionDir;
    const file = p.resolve(base, l.configured);
    consider("codelaw.path", file, true);
  }
  for (const folder of l.folders) {
    consider("node_modules", p.join(folder, "node_modules", ".bin", win ? "codelaw.cmd" : "codelaw"));
    consider("venv", p.join(folder, ".venv", win ? "Scripts" : "bin", exe));
  }
  if (l.python) consider("python", p.join(p.dirname(l.python), exe));
  consider("bundled", p.join(l.extensionDir, "bin", exe));
  return { found, tried };
}
