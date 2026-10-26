//! `--format github|gitlab|junit|checkstyle`: what CI platforms read
//! (SR-CLI-MCP decision 4). Each carries the L4 fix fields where its shape
//! has room: the class and steps in GitLab's `content.body` and in a JUnit
//! `<failure>`'s text, and as a `[fix: …]` tail on the message where the
//! shape has only a message (GitHub, Checkstyle). Nothing here reads a clock,
//! a host or an environment variable, so the same findings give the same
//! bytes (L5).

use crate::counts;
use codelaw_core::{Diagnostic, FixClass, Severity};
use codelaw_engine::CheckReport;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A finding's stable identity: a hash of the rule, the file, the message and
/// the occurrence's index among that rule's findings in the file. Never the
/// line, so moving code does not re-open a finding. Indexed in report order,
/// which is path, line, column, rule (L5 decision 2).
pub fn fingerprints(diagnostics: &[Diagnostic]) -> Vec<String> {
    let mut seen: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    diagnostics
        .iter()
        .map(|d| {
            let n = seen.entry((d.rule.as_str(), d.file.as_str())).or_default();
            let index = *n;
            *n += 1;
            let mut h = blake3::Hasher::new();
            for part in [d.rule.as_str(), d.file.as_str(), d.message.as_str()] {
                h.update(part.as_bytes());
                h.update(&[0]);
            }
            h.update(index.to_string().as_bytes());
            h.finalize().to_hex().as_str()[..32].to_owned()
        })
        .collect()
}

/// The class a reader can act on: a fix with no edits is `manual`.
fn effective_class(d: &Diagnostic) -> &'static str {
    match d.fix {
        FixClass::Auto | FixClass::Suggest if d.edits.is_empty() => "manual",
        c => c.name(),
    }
}

/// `[fix: auto]`, or `[fix: manual — steps]`.
fn fix_tail(d: &Diagnostic) -> String {
    let class = effective_class(d);
    if class == "manual" && !d.steps.is_empty() { format!("[fix: manual — {}]", d.steps.join(" ")) } else { format!("[fix: {class}]") }
}

fn fix_lines(d: &Diagnostic) -> String {
    let mut s = format!("fix: {}", effective_class(d));
    if let Some(n) = &d.note {
        let _ = write!(s, " ({n})");
    }
    for (i, step) in d.steps.iter().enumerate() {
        let _ = write!(s, "\n{}. {step}", i + 1);
    }
    s
}

// ---- GitHub Actions workflow commands ----------------------------------

fn gh_data(s: &str) -> String {
    s.replace('%', "%25").replace('\r', "%0D").replace('\n', "%0A")
}

fn gh_prop(s: &str) -> String {
    gh_data(s).replace(':', "%3A").replace(',', "%2C")
}

/// `::error file=…,line=…,col=…,endLine=…,endColumn=…,title=<rule>::<msg>`
/// per finding (warn is `::warning`, info `::notice`), the run's warnings as
/// bare `::warning::`, and a plain summary line last: Actions shows only
/// about ten annotations of each kind inline.
pub fn check_github(report: &CheckReport) -> String {
    let mut out = String::new();
    for w in &report.warnings {
        let _ = writeln!(out, "::warning::{}", gh_data(w));
    }
    for d in &report.diagnostics {
        let cmd = match d.severity {
            Severity::Error => "error",
            Severity::Warn => "warning",
            Severity::Info => "notice",
        };
        let _ = writeln!(
            out,
            "::{cmd} file={},line={},col={},endLine={},endColumn={},title={}::{}",
            gh_prop(&d.file),
            d.start.line,
            d.start.column,
            d.end.line,
            d.end.column,
            gh_prop(&d.rule),
            gh_data(&format!("{} {}", d.message, fix_tail(d)))
        );
    }
    for s in &report.skipped {
        let _ = writeln!(out, "::notice file={}::{}", gh_prop(&s.file), gh_data(&format!("skipped: {}", s.reason)));
    }
    let c = counts(&report.diagnostics);
    let _ = writeln!(out, "codelaw: {} errors, {} warnings, {} infos in {} files checked", c.errors, c.warnings, c.infos, report.files_checked);
    out
}

// ---- GitLab Code Quality ------------------------------------------------

fn gitlab_severity(s: Severity) -> &'static str {
    match s {
        Severity::Error => "major",
        Severity::Warn => "minor",
        Severity::Info => "info",
    }
}

/// The Code Quality report: a JSON array, one object per finding.
pub fn check_gitlab(report: &CheckReport) -> Value {
    let prints = fingerprints(&report.diagnostics);
    Value::Array(
        report
            .diagnostics
            .iter()
            .zip(prints)
            .map(|(d, fingerprint)| {
                json!({
                    "description": d.message,
                    "check_name": d.rule,
                    "fingerprint": fingerprint,
                    "severity": gitlab_severity(d.severity),
                    "location": { "path": d.file, "lines": { "begin": d.start.line, "end": d.end.line } },
                    "content": { "body": fix_lines(d) },
                })
            })
            .collect(),
    )
}

// ---- XML ----------------------------------------------------------------

/// Escaped for text and attributes; characters XML 1.0 cannot carry are
/// dropped.
fn xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\t' | '\n' | '\r' => out.push(c),
            c if (c as u32) < 0x20 || c == '\u{FFFE}' || c == '\u{FFFF}' => {}
            c => out.push(c),
        }
    }
    out
}

fn by_file(report: &CheckReport) -> BTreeMap<&str, Vec<&Diagnostic>> {
    let mut m: BTreeMap<&str, Vec<&Diagnostic>> = BTreeMap::new();
    for d in &report.diagnostics {
        m.entry(d.file.as_str()).or_default().push(d);
    }
    m
}

/// JUnit XML (the Ant/Jenkins `testsuites` shape): a suite per file with
/// findings, a test case per finding. Error and warn are `<failure>`, info
/// is `<skipped>`. The timestamp, host and time are constants: a report that
/// changed with the clock would not be byte-identical.
pub fn check_junit(report: &CheckReport) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites>\n");
    for (id, (file, ds)) in by_file(report).iter().enumerate() {
        let failures = ds.iter().filter(|d| d.severity != Severity::Info).count();
        let skipped = ds.len() - failures;
        let _ = writeln!(
            out,
            "  <testsuite name=\"{n}\" package=\"{n}\" id=\"{id}\" timestamp=\"1970-01-01T00:00:00\" hostname=\"localhost\" tests=\"{}\" failures=\"{failures}\" errors=\"0\" skipped=\"{skipped}\" time=\"0\">",
            ds.len(),
            n = xml(file)
        );
        out.push_str("    <properties/>\n");
        for d in ds {
            let name = xml(&format!("{}:{}:{}", d.rule, d.start.line, d.start.column));
            let _ = writeln!(out, "    <testcase name=\"{name}\" classname=\"{}\" time=\"0\">", xml(file));
            let body = xml(&format!("{}:{}:{} {}\n{}", d.file, d.start.line, d.start.column, d.message, fix_lines(d)));
            if d.severity == Severity::Info {
                let _ = writeln!(out, "      <skipped message=\"{}\">{body}</skipped>", xml(&d.message));
            } else {
                let _ = writeln!(out, "      <failure message=\"{}\" type=\"{}\">{body}</failure>", xml(&d.message), xml(&d.rule));
            }
            out.push_str("    </testcase>\n");
        }
        out.push_str("    <system-out></system-out>\n    <system-err></system-err>\n  </testsuite>\n");
    }
    out.push_str("</testsuites>\n");
    out
}

/// Checkstyle XML: a `<file>` per file with findings, an `<error>` per
/// finding. `source` is the rule id.
pub fn check_checkstyle(report: &CheckReport) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(out, "<checkstyle version=\"{}\">", xml(codelaw_core::VERSION));
    for (file, ds) in by_file(report) {
        let _ = writeln!(out, "  <file name=\"{}\">", xml(file));
        for d in ds {
            let severity = match d.severity {
                Severity::Error => "error",
                Severity::Warn => "warning",
                Severity::Info => "info",
            };
            let _ = writeln!(
                out,
                "    <error line=\"{}\" column=\"{}\" severity=\"{severity}\" message=\"{}\" source=\"{}\"/>",
                d.start.line,
                d.start.column,
                xml(&format!("{} {}", d.message, fix_tail(d))),
                xml(&d.rule)
            );
        }
        out.push_str("  </file>\n");
    }
    out.push_str("</checkstyle>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use codelaw_core::Position;

    fn d(rule: &str, file: &str, line: u32, message: &str) -> Diagnostic {
        Diagnostic {
            rule: rule.into(),
            severity: Severity::Error,
            file: file.into(),
            start_byte: 0,
            end_byte: 1,
            start: Position { line, column: 1 },
            end: Position { line, column: 2 },
            message: message.into(),
            fix: FixClass::Manual,
            edits: vec![],
            steps: vec!["Do it.".into()],
            source: "local".into(),
            note: None,
            data: None,
        }
    }

    #[test]
    fn fingerprints_ignore_the_line_and_tell_repeats_apart() {
        let a = fingerprints(&[d("r/x", "a.js", 3, "m"), d("r/x", "a.js", 9, "m"), d("r/y", "a.js", 9, "m")]);
        let b = fingerprints(&[d("r/x", "a.js", 30, "m"), d("r/x", "a.js", 90, "m"), d("r/y", "a.js", 90, "m")]);
        assert_eq!(a, b);
        assert_ne!(a[0], a[1]);
        assert_ne!(a[1], a[2]);
        assert_eq!(a[0].len(), 32);
        assert_ne!(fingerprints(&[d("r/x", "b.js", 3, "m")])[0], a[0]);
        assert_ne!(fingerprints(&[d("r/x", "a.js", 3, "other")])[0], a[0]);
    }

    #[test]
    fn github_escapes_and_maps_severity() {
        let mut w = d("r/w", "a,b:c.js", 1, "100%\nnext");
        w.severity = Severity::Warn;
        let r = CheckReport { diagnostics: vec![w], files_checked: 2, ..Default::default() };
        let t = check_github(&r);
        assert!(t.starts_with("::warning file=a%2Cb%3Ac.js,line=1,col=1,endLine=1,endColumn=2,title=r/w::100%25%0Anext [fix: manual — Do it.]\n"), "{t}");
        assert!(t.ends_with("codelaw: 0 errors, 1 warnings, 0 infos in 2 files checked\n"), "{t}");
    }
}
