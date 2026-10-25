// Types for the codelaw npm API (SR-PACKAGES decision 3). Each call runs the
// bundled binary and returns its `--format json` report unchanged.

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

export interface FormatOptions extends Options {
  /** `--check`: report what would be formatted, write nothing. */
  check?: boolean;
}

export interface FixReport extends CheckReport {
  /** Each changed file, the fixes applied per rule, and whether it was written. */
  fixed: { file: string; applied: Record<string, number>; passes: number; written: boolean }[];
}

export interface FormatReport extends CheckReport {
  check: boolean;
  /** Each file whose layout changed; `written` once the disk holds it. */
  formatted: { file: string; written: boolean }[];
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
  /** A security rule's CWE ids, OWASP Top 10 categories and severity (SR-RULE-FORMAT decision 2a). */
  security?: {
    cwe: number[];
    owasp: string[];
    severity: "high" | "medium" | "low";
    /** SARIF's `security-severity` score for `severity`. */
    security_severity: string;
  };
}

export interface RulesReport {
  schema_version: number;
  codelaw_version: string;
  /** The file asked about; `null` in a listing of every rule. */
  for: string | null;
  rules: RuleCard[];
  [key: string]: unknown;
}

export const PLATFORMS: readonly string[];
export function platformKey(): string;
export function binaryPath(): string;
export function check(paths?: string[], options?: Options): CheckReport;
export function fix(paths?: string[], options?: FixOptions): FixReport;
export function format(paths?: string[], options?: FormatOptions): FormatReport;
export function rulesFor(path: string): RulesReport;
