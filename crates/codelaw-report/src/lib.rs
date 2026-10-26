//! Output formats and exit codes (SR-CLI-MCP decisions 3 and 4). Every
//! format carries the L4 fix fields: the class, and the edits or the steps.
//! SARIF is `sarif.rs`; the debug commands' output is `debug.rs`.

mod ci;
mod debug;
pub mod exceptions;
mod sarif;

pub use ci::{check_checkstyle, check_github, check_gitlab, check_junit, fingerprints};
pub use debug::{config_json, config_text, match_json, match_text, timings_json, timings_text, trace_json, trace_text};
pub use sarif::{SARIF_SCHEMA, check_sarif};

mod hook;
mod query;

pub use hook::{HOOK_FINDINGS, hook_blocks, hook_text};

pub use query::{
    explain_engine_json, explain_engine_text, explain_json, explain_text, fix_skill, rules_json, rules_text, security_coverage_json, security_coverage_text,
    security_json, snippet_fix_json, tree_json, tree_text, why_json, why_text,
};

use codelaw_core::{Diagnostic, FixClass, Severity};
use codelaw_engine::{CheckReport, FixReport, FormatReport, Project};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The JSON output's version: bumped on any breaking change to its shape
/// (decision 9).
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
    Sarif,
    Agent,
    Github,
    Gitlab,
    Junit,
    Checkstyle,
}

impl Format {
    pub fn parse(s: &str) -> Option<Format> {
        Some(match s {
            "text" => Format::Text,
            "json" => Format::Json,
            "sarif" => Format::Sarif,
            "agent" => Format::Agent,
            "github" => Format::Github,
            "gitlab" => Format::Gitlab,
            "junit" => Format::Junit,
            "checkstyle" => Format::Checkstyle,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub fixable: usize,
    pub suggestions: usize,
}

pub fn counts(diagnostics: &[Diagnostic]) -> Counts {
    let mut c = Counts::default();
    for d in diagnostics {
        match d.severity {
            Severity::Error => c.errors += 1,
            Severity::Warn => c.warnings += 1,
            Severity::Info => c.infos += 1,
        }
        match d.fix {
            FixClass::Auto if !d.edits.is_empty() => c.fixable += 1,
            FixClass::Suggest => c.suggestions += 1,
            _ => {}
        }
    }
    c
}

/// `0` no errors · `1` errors remain · `2` a rule failed (decision 3;
/// SR-ARCHITECTURE decision 25). Warnings fail only past `--max-warnings`.
pub fn exit_code(report: &CheckReport, max_warnings: Option<usize>) -> i32 {
    if report.internal_errors() > 0 {
        return 2;
    }
    let c = counts(&report.diagnostics);
    if c.errors > 0 || max_warnings.is_some_and(|m| c.warnings > m) {
        1
    } else {
        0
    }
}

/// One diagnostic as `--format json` writes it.
pub fn diagnostic_json(d: &Diagnostic) -> Value {
    let mut v = json!({
        "rule": d.rule,
        "severity": d.severity.name(),
        "file": d.file,
        "range": {
            "start": { "line": d.start.line, "column": d.start.column, "offset": d.start_byte },
            "end": { "line": d.end.line, "column": d.end.column, "offset": d.end_byte },
        },
        "message": d.message,
        "fix": {
            "class": d.fix.name(),
            "edits": d.edits.iter().map(|e| json!({ "range": { "start": e.start, "end": e.end }, "text": e.text })).collect::<Vec<_>>(),
            "steps": d.steps,
            "note": d.note,
        },
        "source": d.source,
    });
    // A size rule's measured value and its limit (`{ "lines": 412, "max": 300 }`).
    if let Some(data) = &d.data {
        v["data"] = data.clone();
    }
    v
}

fn summary_json(report: &CheckReport) -> Value {
    let c = counts(&report.diagnostics);
    json!({
        "files_checked": report.files_checked,
        "files_skipped": report.skipped.len(),
        "errors": c.errors,
        "warnings": c.warnings,
        "infos": c.infos,
        "fixable": c.fixable,
        "suggestions": c.suggestions,
        "internal_errors": report.internal_errors(),
    })
}

/// The `--format json` object for `check`.
pub fn check_json(report: &CheckReport) -> Value {
    let mut v = json!({
        "schema_version": SCHEMA_VERSION,
        "codelaw_version": codelaw_core::VERSION,
        "summary": summary_json(report),
        "diagnostics": report.diagnostics.iter().map(diagnostic_json).collect::<Vec<_>>(),
        "skipped": report.skipped.iter().map(|s| json!({ "file": s.file, "reason": s.reason })).collect::<Vec<_>>(),
        "warnings": report.warnings,
    });
    // Only when a baseline or a deprecation was in play (SR-BASELINE
    // decision 4): the date deadlines were judged on, and its source.
    if let Some(a) = &report.as_of {
        v["as_of"] = json!({ "date": a.date.map(|d| d.to_string()), "source": a.source });
    }
    // Only when the config defines a profile (SR-CONFIG decision 10).
    if let Some(p) = &report.profile {
        v["profile"] = json!({ "name": p.name, "source": p.source });
    }
    // Only when the test rules of new code ran (W-71).
    if let Some(n) = &report.new_code {
        v["new_code"] = json!({ "change_set": n.change_set, "coverage": n.coverage });
    }
    v
}

/// The `--format json` object for `fix`: the check shape plus what changed.
pub fn fix_json(report: &FixReport) -> Value {
    let mut v = check_json(&report.check);
    v["fixed"] = report
        .files
        .iter()
        .map(|f| json!({ "file": f.file, "applied": f.applied, "passes": f.passes, "written": f.written }))
        .collect::<Vec<_>>()
        .into();
    if !report.pruned.is_empty() {
        v["pruned"] = json!(report.pruned);
    }
    v
}

/// The `--format json` object for `format`: the check shape (files
/// formatted, skipped, `format/failed` and write failures as diagnostics)
/// plus `formatted[]`, each file whose layout changed — `written` once the
/// disk holds it, `false` with `--check`.
pub fn format_json(report: &FormatReport, check: bool) -> Value {
    let mut v = check_json(&report.check);
    v["check"] = check.into();
    v["formatted"] = report.files.iter().map(|f| json!({ "file": f.file, "written": f.written })).collect::<Vec<_>>().into();
    v
}

struct Paint(bool);

impl Paint {
    fn wrap(&self, code: &str, s: &str) -> String {
        if self.0 { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_owned() }
    }
    fn dim(&self, s: &str) -> String {
        self.wrap("2", s)
    }
    fn bold(&self, s: &str) -> String {
        self.wrap("1", s)
    }
    fn severity(&self, s: Severity) -> String {
        let code = match s {
            Severity::Error => "31",
            Severity::Warn => "33",
            Severity::Info => "36",
        };
        self.wrap(code, s.name())
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn summary_line(report: &CheckReport, paint: &Paint, fixed: Option<usize>) -> String {
    let c = counts(&report.diagnostics);
    let problems = c.errors + c.warnings + c.infos;
    let mut parts = vec![format!(
        "{} ({}, {}, {})",
        plural(problems, "problem", "problems"),
        plural(c.errors, "error", "errors"),
        plural(c.warnings, "warning", "warnings"),
        plural(c.infos, "info", "infos")
    )];
    if let Some(n) = fixed {
        parts.push(format!("{} fixed", plural(n, "file", "files")));
    }
    if c.fixable > 0 {
        parts.push(format!("{} with `codelaw fix`", plural(c.fixable, "auto fix", "auto fixes")));
    }
    if c.suggestions > 0 {
        parts.push(format!("{} with `codelaw fix --unsafe`", plural(c.suggestions, "suggestion", "suggestions")));
    }
    parts.push(format!("{} checked", plural(report.files_checked, "file", "files")));
    // Partial results are never silently partial (SR-ARCHITECTURE decision 26).
    if !report.skipped.is_empty() {
        parts.push(format!("{} skipped", plural(report.skipped.len(), "file", "files")));
    }
    if report.internal_errors() > 0 {
        parts.push(format!("{} failed", plural(report.internal_errors(), "rule run", "rule runs")));
    }
    let head = if c.errors > 0 { paint.wrap("31;1", "✖") } else { paint.wrap("32;1", "✔") };
    format!("{head} {}", parts.join(" · "))
}

/// What text and agent output print after a rule id: a security rule's CWE
/// ids, `(CWE-78)` (SR-CLI-MCP decision 4).
pub type Labels = BTreeMap<String, String>;

/// The labels of every rule of the project that has them.
pub fn security_labels(project: &Project) -> Labels {
    project
        .rules
        .rules
        .iter()
        .filter_map(|r| r.rule.security.as_ref().map(|s| (r.rule.id.clone(), s.cwe_label())))
        .filter(|(_, l)| !l.is_empty())
        .collect()
}

fn labelled(rule: &str, labels: &Labels) -> String {
    match labels.get(rule) {
        Some(l) => format!("{rule} {l}"),
        None => rule.to_owned(),
    }
}

/// Grouped by file, coloured on a TTY.
pub fn check_text(report: &CheckReport, color: bool) -> String {
    text(report, color, None, &Labels::new())
}

/// [`check_text`], each security rule's id followed by its CWE ids.
pub fn check_text_with(report: &CheckReport, color: bool, labels: &Labels) -> String {
    text(report, color, None, labels)
}

pub fn fix_text(report: &FixReport, color: bool) -> String {
    text(&report.check, color, Some(report.fixed_count()), &Labels::new())
}

pub fn fix_text_with(report: &FixReport, color: bool, labels: &Labels) -> String {
    text(&report.check, color, Some(report.fixed_count()), labels)
}

fn text(report: &CheckReport, color: bool, fixed: Option<usize>, labels: &Labels) -> String {
    let paint = Paint(color);
    let mut out = String::new();
    if let Some(p) = &report.profile {
        out.push_str(&format!("{}\n", p.line()));
    }
    if let Some(a) = &report.as_of {
        out.push_str(&format!("{}\n", a.line()));
    }
    if let Some(n) = &report.new_code {
        out.push_str(&format!("{}\n", n.change_set));
        if let Some(c) = &n.coverage {
            out.push_str(&format!("{c}\n"));
        }
    }
    for w in &report.warnings {
        out.push_str(&format!("{} {w}\n", paint.wrap("33", "warning:")));
    }
    let mut by_file: BTreeMap<&str, Vec<&Diagnostic>> = BTreeMap::new();
    for d in &report.diagnostics {
        by_file.entry(d.file.as_str()).or_default().push(d);
    }
    for (file, ds) in &by_file {
        out.push_str(&format!("\n{}\n", paint.bold(file)));
        for d in ds {
            let fix = match d.fix {
                FixClass::Auto if !d.edits.is_empty() => "auto".to_owned(),
                FixClass::Suggest if !d.edits.is_empty() => "suggest".to_owned(),
                class => class.name().to_owned(),
            };
            let note = d.note.as_deref().map(|n| format!(" ({n})")).unwrap_or_default();
            out.push_str(&format!(
                "  {}  {}  {}  {}\n",
                paint.dim(&format!("{}:{}", d.start.line, d.start.column)),
                paint.severity(d.severity),
                d.message,
                paint.dim(&format!("{} [{fix}]{note}", labelled(&d.rule, labels)))
            ));
            if d.fix == FixClass::Manual {
                for (i, s) in d.steps.iter().enumerate() {
                    out.push_str(&format!("      {} {s}\n", paint.dim(&format!("{}.", i + 1))));
                }
            }
        }
    }
    for s in &report.skipped {
        out.push_str(&format!("{} {}: {}\n", paint.dim("skipped"), s.file, s.reason));
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&summary_line(report, &paint, fixed));
    out.push('\n');
    out
}

/// Compact lines for hooks: one per diagnostic, with what to do next.
pub fn check_agent(report: &CheckReport) -> String {
    check_agent_with(report, &Labels::new())
}

/// [`check_agent`], each security rule's id followed by its CWE ids.
pub fn check_agent_with(report: &CheckReport, labels: &Labels) -> String {
    let mut out = String::new();
    if let Some(p) = &report.profile {
        out.push_str(&format!("{}\n", p.line()));
    }
    if let Some(a) = &report.as_of {
        out.push_str(&format!("{}\n", a.line()));
    }
    if let Some(n) = &report.new_code {
        out.push_str(&format!("{}\n", n.change_set));
        if let Some(c) = &n.coverage {
            out.push_str(&format!("{c}\n"));
        }
    }
    for d in &report.diagnostics {
        let next = match d.fix {
            FixClass::Auto if !d.edits.is_empty() => "fix: auto (`codelaw fix`)".to_owned(),
            FixClass::Suggest if !d.edits.is_empty() => "fix: suggest (diff in --format json)".to_owned(),
            _ => format!("fix: manual — {}", d.steps.join(" ")),
        };
        out.push_str(&format!("{}:{}:{} {} {} — {next}\n", d.file, d.start.line, d.start.column, labelled(&d.rule, labels), d.message));
    }
    for s in &report.skipped {
        out.push_str(&format!("{} skipped: {}\n", s.file, s.reason));
    }
    let c = counts(&report.diagnostics);
    out.push_str(&format!("{} errors, {} warnings, {} infos\n", c.errors, c.warnings, c.infos));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use codelaw_core::{Position, Skipped, TextEdit};

    fn diag(rule: &str, severity: Severity, fix: FixClass) -> Diagnostic {
        Diagnostic {
            rule: rule.into(),
            severity,
            file: "src/a.js".into(),
            start_byte: 0,
            end_byte: 6,
            start: Position { line: 1, column: 1 },
            end: Position { line: 1, column: 7 },
            message: "Use ===".into(),
            fix,
            edits: if fix == FixClass::Manual { vec![] } else { vec![TextEdit::new(2, 4, "===")] },
            steps: if fix == FixClass::Manual { vec!["Do it.".into()] } else { vec![] },
            source: "local".into(),
            note: None,
            data: None,
        }
    }

    #[test]
    fn exit_codes() {
        let mut r = CheckReport::default();
        assert_eq!(exit_code(&r, None), 0);
        r.diagnostics.push(diag("a/w", Severity::Warn, FixClass::Auto));
        assert_eq!(exit_code(&r, None), 0);
        assert_eq!(exit_code(&r, Some(0)), 1);
        r.diagnostics.push(diag("a/e", Severity::Error, FixClass::Manual));
        assert_eq!(exit_code(&r, None), 1);
        r.diagnostics.push(diag("codelaw/internal-error", Severity::Error, FixClass::Manual));
        assert_eq!(exit_code(&r, None), 2);
    }

    #[test]
    fn json_carries_the_fix_fields() {
        let r = CheckReport {
            diagnostics: vec![diag("a/e", Severity::Error, FixClass::Auto), diag("a/m", Severity::Warn, FixClass::Manual)],
            skipped: vec![Skipped { file: "x.js".into(), reason: "not valid UTF-8".into() }],
            files_checked: 1,
            warnings: vec![],
            as_of: None,
            new_code: None,
            ..Default::default()
        };
        let v = check_json(&r);
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["summary"]["errors"], 1);
        assert_eq!(v["summary"]["files_skipped"], 1);
        assert_eq!(v["diagnostics"][0]["fix"]["class"], "auto");
        assert_eq!(v["diagnostics"][0]["fix"]["edits"][0]["text"], "===");
        assert_eq!(v["diagnostics"][1]["fix"]["steps"][0], "Do it.");
        assert_eq!(v["skipped"][0]["reason"], "not valid UTF-8");
    }

    #[test]
    fn json_carries_a_findings_data_only_when_it_has_some() {
        let mut with = diag("js/max-lines", Severity::Warn, FixClass::Manual);
        with.data = Some(json!({ "lines": 412, "max": 300 }));
        assert_eq!(diagnostic_json(&with)["data"], json!({ "lines": 412, "max": 300 }));
        assert!(diagnostic_json(&diag("a/m", Severity::Warn, FixClass::Manual)).get("data").is_none());
    }

    #[test]
    fn text_summary_names_skips() {
        let r = CheckReport {
            diagnostics: vec![diag("a/e", Severity::Error, FixClass::Auto)],
            skipped: vec![Skipped { file: "x.js".into(), reason: "r".into() }],
            files_checked: 1,
            warnings: vec![],
            as_of: None,
            new_code: None,
            ..Default::default()
        };
        let t = check_text(&r, false);
        assert!(t.contains("src/a.js\n  1:1  error  Use ===  a/e [auto]"), "{t}");
        assert!(t.ends_with("✖ 1 problem (1 error, 0 warnings, 0 infos) · 1 auto fix with `codelaw fix` · 1 file checked · 1 file skipped\n"), "{t}");
    }
}
