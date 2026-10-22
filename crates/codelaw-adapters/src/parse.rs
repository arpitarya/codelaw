//! Reading each tool's JSON report (SR-ARCHITECTURE decision 19) and
//! mapping a finding onto a codelaw diagnostic with its fix class (L4
//! decision 9).

use crate::manifest::{Manifest, Output};
use crate::position::{self, Unit};
use codelaw_core::{Diagnostic, FixClass, LineIndex, Severity, TextEdit};
use serde::Deserialize;

/// Where a finding or an edit is, in the tool's own units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Span {
    /// Whole-file offsets, end exclusive.
    Offsets { start: usize, end: usize, unit: Unit },
    /// 1-based lines and columns, end exclusive; no end is an empty range.
    Lines { start: (u32, u32), end: Option<(u32, u32)>, unit: Unit },
}

impl Span {
    /// The byte range in `text`.
    pub fn bytes(&self, text: &str) -> (usize, usize) {
        let (s, e) = match *self {
            Span::Offsets { start, end, unit } => (position::offset(text, start, unit), position::offset(text, end, unit)),
            Span::Lines { start, end, unit } => {
                let s = position::line_column(text, start.0, start.1, unit);
                (s, end.map_or(s, |(l, c)| position::line_column(text, l, c, unit)))
            }
        };
        (s, e.max(s))
    }
}

/// One edit of a tool's own fix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawEdit {
    pub span: Span,
    pub text: String,
}

/// One finding as the tool reported it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// The file as the tool names it: relative to the working directory
    /// or absolute.
    pub file: String,
    /// The tool's rule, e.g. `F401` or `eslint/no-debugger`.
    pub rule: String,
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub fix: FixClass,
    pub edits: Vec<RawEdit>,
    /// The tool's help line, if it gives one.
    pub help: Option<String>,
    /// The rule's docs, if the report links them.
    pub url: Option<String>,
}

/// A whole report: the findings, and what the tool warned about its own
/// configuration.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parsed {
    pub findings: Vec<Finding>,
    pub warnings: Vec<String>,
}

/// Read a report. stylelint prints it on stderr when there are problems,
/// so the stream that holds JSON is the report.
pub fn parse(output: Output, stdout: &str, stderr: &str) -> Result<Parsed, String> {
    let json = |s: &str| s.trim_start().starts_with(['[', '{']);
    let text = if json(stdout) || !json(stderr) { stdout } else { stderr };
    match output {
        Output::Oxlint => oxlint(text),
        Output::Ruff => ruff(text),
        Output::Stylelint => stylelint(text),
        Output::HtmlValidate => html_validate(text),
    }
}

fn invalid(e: serde_json::Error) -> String {
    format!("the report is not the JSON this adapter reads: {e}")
}

fn severity(s: &str) -> Severity {
    match s {
        "error" => Severity::Error,
        "warning" | "warn" => Severity::Warn,
        _ => Severity::Info,
    }
}

// --- oxlint: `--format=json` ---------------------------------------------

#[derive(Deserialize)]
struct OxReport {
    diagnostics: Vec<OxDiagnostic>,
}

#[derive(Deserialize)]
struct OxDiagnostic {
    message: String,
    #[serde(default)]
    code: Option<String>,
    severity: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    help: Option<String>,
    filename: String,
    #[serde(default)]
    labels: Vec<OxLabel>,
}

#[derive(Deserialize)]
struct OxLabel {
    span: OxSpan,
}

#[derive(Deserialize)]
struct OxSpan {
    offset: usize,
    length: usize,
}

/// `eslint(no-debugger)` is `eslint/no-debugger`; a diagnostic with no code
/// is a parse error.
fn oxlint_rule(code: Option<&str>) -> String {
    match code.filter(|c| !c.is_empty()) {
        None => "parse-error".to_owned(),
        Some(c) => match c.split_once('(') {
            Some((plugin, rule)) => format!("{plugin}/{}", rule.trim_end_matches(')')),
            None => c.to_owned(),
        },
    }
}

fn oxlint(text: &str) -> Result<Parsed, String> {
    let report: OxReport = serde_json::from_str(text).map_err(invalid)?;
    let findings = report
        .diagnostics
        .into_iter()
        .map(|d| {
            let span = d.labels.first().map_or(Span::Offsets { start: 0, end: 0, unit: Unit::Byte }, |l| Span::Offsets {
                start: l.span.offset,
                end: l.span.offset + l.span.length,
                unit: Unit::Byte,
            });
            Finding {
                file: d.filename,
                rule: oxlint_rule(d.code.as_deref()),
                severity: severity(&d.severity),
                message: d.message,
                span,
                fix: FixClass::Manual,
                edits: Vec::new(),
                help: d.help.filter(|h| !h.is_empty()),
                url: d.url.filter(|u| !u.is_empty()),
            }
        })
        .collect();
    Ok(Parsed { findings, warnings: Vec::new() })
}

// --- Ruff: `check --output-format=json` ----------------------------------

#[derive(Deserialize)]
struct RuffLocation {
    row: u32,
    column: u32,
}

#[derive(Deserialize)]
struct RuffEdit {
    content: String,
    location: RuffLocation,
    end_location: RuffLocation,
}

#[derive(Deserialize)]
struct RuffFix {
    applicability: String,
    edits: Vec<RuffEdit>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize)]
struct RuffDiagnostic {
    #[serde(default)]
    code: Option<String>,
    location: RuffLocation,
    end_location: RuffLocation,
    filename: String,
    #[serde(default)]
    fix: Option<RuffFix>,
    message: String,
    #[serde(default)]
    severity: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

fn ruff(text: &str) -> Result<Parsed, String> {
    let report: Vec<RuffDiagnostic> = serde_json::from_str(text).map_err(invalid)?;
    let at = |l: &RuffLocation| (l.row, l.column);
    let findings = report
        .into_iter()
        .map(|d| {
            // L4 decision 9: the tool's safety flag decides the class.
            let (fix, edits, help) = match d.fix {
                Some(f) => {
                    let class = match f.applicability.as_str() {
                        "safe" => FixClass::Auto,
                        "unsafe" => FixClass::Suggest,
                        _ => FixClass::Manual,
                    };
                    let edits = if class == FixClass::Manual {
                        Vec::new()
                    } else {
                        f.edits
                            .iter()
                            .map(|e| RawEdit {
                                span: Span::Lines { start: at(&e.location), end: Some(at(&e.end_location)), unit: Unit::CodePoint },
                                text: e.content.clone(),
                            })
                            .collect()
                    };
                    (class, edits, f.message)
                }
                None => (FixClass::Manual, Vec::new(), None),
            };
            Finding {
                file: d.filename,
                // A syntax error has no code.
                rule: d.code.filter(|c| !c.is_empty()).unwrap_or_else(|| "syntax-error".to_owned()),
                severity: d.severity.as_deref().map_or(Severity::Error, severity),
                message: d.message,
                span: Span::Lines { start: at(&d.location), end: Some(at(&d.end_location)), unit: Unit::CodePoint },
                fix,
                edits,
                help,
                url: d.url.filter(|u| !u.is_empty()),
            }
        })
        .collect();
    Ok(Parsed { findings, warnings: Vec::new() })
}

// --- stylelint: `--formatter json --compute-edit-info` -------------------

#[derive(Deserialize)]
struct StyleResult {
    source: Option<String>,
    #[serde(default)]
    warnings: Vec<StyleWarning>,
    #[serde(default, rename = "invalidOptionWarnings")]
    invalid_option_warnings: Vec<StyleText>,
    #[serde(default)]
    deprecations: Vec<StyleText>,
}

#[derive(Deserialize)]
struct StyleText {
    text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StyleWarning {
    line: u32,
    column: u32,
    #[serde(default)]
    end_line: Option<u32>,
    #[serde(default)]
    end_column: Option<u32>,
    rule: String,
    severity: String,
    text: String,
    #[serde(default)]
    fix: Option<StyleFix>,
}

#[derive(Deserialize)]
struct StyleFix {
    range: [usize; 2],
    text: String,
}

fn stylelint(text: &str) -> Result<Parsed, String> {
    let report: Vec<StyleResult> = serde_json::from_str(text).map_err(invalid)?;
    let mut parsed = Parsed::default();
    for r in report {
        for w in r.invalid_option_warnings.iter().chain(&r.deprecations) {
            if !parsed.warnings.contains(&w.text) {
                parsed.warnings.push(w.text.clone());
            }
        }
        let Some(file) = r.source else { continue };
        for w in r.warnings {
            let end = w.end_line.zip(w.end_column);
            let (fix, edits) = match w.fix {
                Some(f) => (FixClass::Auto, vec![RawEdit { span: Span::Offsets { start: f.range[0], end: f.range[1], unit: Unit::Utf16 }, text: f.text }]),
                None => (FixClass::Manual, Vec::new()),
            };
            // The text ends with ` (rule)`; the rule is in its own field.
            let suffix = format!(" ({})", w.rule);
            let message = w.text.strip_suffix(&suffix).unwrap_or(&w.text).to_owned();
            parsed.findings.push(Finding {
                file: file.clone(),
                rule: w.rule,
                severity: severity(&w.severity),
                message,
                span: Span::Lines { start: (w.line, w.column), end, unit: Unit::Utf16 },
                fix,
                edits,
                help: None,
                url: None,
            });
        }
    }
    Ok(parsed)
}

// --- html-validate: `--formatter json` -----------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HvResult {
    file_path: String,
    #[serde(default)]
    messages: Vec<HvMessage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HvMessage {
    #[serde(default)]
    rule_id: Option<String>,
    severity: u8,
    message: String,
    offset: usize,
    #[serde(default)]
    size: usize,
    #[serde(default)]
    rule_url: Option<String>,
}

fn html_validate(text: &str) -> Result<Parsed, String> {
    let report: Vec<HvResult> = serde_json::from_str(text).map_err(invalid)?;
    let mut parsed = Parsed::default();
    for r in report {
        for m in r.messages {
            parsed.findings.push(Finding {
                file: r.file_path.clone(),
                rule: m.rule_id.filter(|id| !id.is_empty()).unwrap_or_else(|| "parser-error".to_owned()),
                severity: if m.severity >= 2 { Severity::Error } else { Severity::Warn },
                message: m.message,
                span: Span::Offsets { start: m.offset, end: m.offset + m.size, unit: Unit::Utf16 },
                fix: FixClass::Manual,
                edits: Vec::new(),
                help: None,
                url: m.rule_url.filter(|u| !u.is_empty()),
            });
        }
    }
    Ok(parsed)
}

// --- the mapping ---------------------------------------------------------

impl Finding {
    /// The rule's docs: the report's link, else the manifest's template.
    pub fn docs(&self, manifest: &Manifest) -> Option<String> {
        self.url.clone().or_else(|| manifest.docs.as_ref().map(|d| d.replace("{rule}", &self.rule)))
    }

    /// The codelaw diagnostic for this finding in `text` (the file without
    /// its byte-order mark), at `rel`. `source` is `<adapter>@<version>`.
    /// An `auto` or `suggest` finding carries the tool's edits; a `manual`
    /// one carries steps that link the rule's docs (L4 decision 9).
    pub fn to_diagnostic(&self, manifest: &Manifest, rel: &str, text: &str, source: &str) -> Diagnostic {
        let (start_byte, end_byte) = self.span.bytes(text);
        let mut edits: Vec<TextEdit> = self
            .edits
            .iter()
            .map(|e| {
                let (s, end) = e.span.bytes(text);
                TextEdit::new(s, end, e.text.clone())
            })
            .collect();
        edits.sort();
        let mut fix = self.fix;
        // Edits that overlap can't be applied as one fix.
        if edits.windows(2).any(|w| w[1].start < w[0].end) || (fix != FixClass::Manual && edits.is_empty()) {
            fix = FixClass::Manual;
        }
        let mut steps = Vec::new();
        if fix == FixClass::Manual {
            edits.clear();
            steps.extend(self.help.clone());
            steps.push(match self.docs(manifest) {
                Some(url) => format!("Change the code by hand; {url} explains `{}`.", self.rule),
                None => format!("Change the code by hand; the {} documentation explains `{}`.", manifest.name, self.rule),
            });
        }
        let index = LineIndex::new(text);
        Diagnostic {
            rule: manifest.rule_id(&self.rule),
            severity: self.severity,
            file: rel.to_owned(),
            start_byte,
            end_byte,
            start: index.position(start_byte),
            end: index.position(end_byte),
            message: self.message.clone(),
            fix,
            edits,
            steps,
            source: source.to_owned(),
            note: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oxlint_codes_become_plugin_and_rule() {
        assert_eq!(oxlint_rule(Some("eslint(no-debugger)")), "eslint/no-debugger");
        assert_eq!(oxlint_rule(Some("typescript-eslint(no-explicit-any)")), "typescript-eslint/no-explicit-any");
        assert_eq!(oxlint_rule(None), "parse-error");
    }

    #[test]
    fn the_stream_with_json_is_the_report() {
        let p = parse(Output::HtmlValidate, "", "[]").unwrap();
        assert!(p.findings.is_empty());
        assert!(parse(Output::Ruff, "error: no such file", "").is_err());
    }
}
