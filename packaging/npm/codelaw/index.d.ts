// Types for the codelaw npm API (SR-PACKAGES decision 3). Each call runs the
// bundled binary and returns its `--format json` report unchanged.
// `format` joins when the formatter does (W-07).

export type Severity = "error" | "warn" | "info";
export type FixClass = "auto" | "suggest" | "manual";

export interface Position { line: number; column: number; offset: number }
export interface Range { start: Position; end: Position }
export interface TextEdit { range: Range; text: string }

export interface Diagnostic {
  rule: string;
  severity: Severity;
  file: string;
  range: Range;
  message: string;
  fix: { class: FixClass; edits: TextEdit[]; steps: string[]; note: string | null } | null;
  source: string;
  [key: string]: unknown;
}

export interface CheckReport {
  schema_version: number;
  codelaw_version: string;
  summary: {
    files_checked: number;
    files_skipped: number;
    errors: number;
    warnings: number;
    infos: number;
    fixable: number;
    suggestions: number;
    internal_errors: number;
  };
  diagnostics: Diagnostic[];
  skipped: unknown[];
  warnings: unknown[];
  [key: string]: unknown;
}

export interface Options {
  /** `--lang`: only these languages. */
  lang?: string[];
  /** `--rule`: only these rules. */
  rule?: string[];
  /** `--threads`. */
  threads?: number;
  /** `--no-cache`. */
  noCache?: boolean;
}

export interface FixOptions extends Options {
  /** `--unsafe`: apply `suggest` fixes too. */
  unsafe?: boolean;
  /** `--dry-run`: compute the fixes, write nothing. */
  dryRun?: boolean;
}

/** One rule as `rules --for` shows it (SR-CLI-MCP decision 7). */
export interface RuleCard {
  id: string;
  title: string;
  severity: Severity | "off";
  fix: FixClass;
  /** `local`, or the pack the rule comes from. */
  source: string;
  preview: boolean;
}

export interface RulesReport {
  schema_version: number;
  codelaw_version: string;
  file: string | null;
  rules: RuleCard[];
  [key: string]: unknown;
}

export const PLATFORMS: readonly string[];
export function platformKey(): string;
export function binaryPath(): string;
export function check(paths?: string[], options?: Options): CheckReport;
export function fix(paths?: string[], options?: FixOptions): Record<string, unknown>;
export function rulesFor(path: string): RulesReport;
