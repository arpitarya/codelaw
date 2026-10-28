//! `--format github|gitlab|junit|checkstyle|bitbucket`: what CI platforms
//! read (SR-CLI-MCP decision 4). Each carries the L4 fix fields where its
//! shape has room: the class and steps in GitLab's `content.body`, in a JUnit
//! `<failure>`'s text and in a Code Insights annotation's `details`, and as a
//! `[fix: …]` tail on the message where the shape has only a message (GitHub,
//! Checkstyle, an annotation's `summary`). Nothing here reads a clock,
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

// ---- Bitbucket Code Insights --------------------------------------------

/// Code Insights' caps (Bitbucket Cloud): annotations per report, per
/// `POST …/annotations`, and the lengths of `summary` and `details`.
pub const BITBUCKET_MAX_ANNOTATIONS: usize = 1000;
pub const BITBUCKET_BATCH: usize = 100;
const BITBUCKET_SUMMARY: usize = 450;
const BITBUCKET_DETAILS: usize = 2000;

/// At most `max` characters, `…` last when cut.
fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max { s.to_owned() } else { s.chars().take(max - 1).chain(['…']).collect() }
}

/// The Code Insights upload, one JSON document per line (W-109): line 1 is
/// the report (`PUT …/commit/<sha>/reports/codelaw`), each line after it is
/// one `POST …/reports/codelaw/annotations` body — an array of at most 100.
/// Errors come first, so a run past the 1000-annotation cap keeps the ones
/// that fail the build; the report's `details` says how many were left out.
pub fn check_bitbucket(report: &CheckReport) -> String {
    let c = counts(&report.diagnostics);
    let total = report.diagnostics.len();
    let mut order: Vec<usize> = (0..total).collect();
    let rank = |s: Severity| match s {
        Severity::Error => 0,
        Severity::Warn => 1,
        Severity::Info => 2,
    };
    order.sort_by_key(|&i| rank(report.diagnostics[i].severity));
    let shown = total.min(BITBUCKET_MAX_ANNOTATIONS);
    let mut details = format!("{} errors, {} warnings, {} infos in {} files checked.", c.errors, c.warnings, c.infos, report.files_checked);
    if shown < total {
        let _ = write!(details, " Showing {shown} of {total} findings: Code Insights holds {BITBUCKET_MAX_ANNOTATIONS} annotations per report; run `codelaw check` for all of them.");
    }
    for w in &report.warnings {
        let _ = write!(details, " Warning: {w}");
    }
    let head = json!({
        "title": "codelaw",
        "details": clip(&details, BITBUCKET_DETAILS),
        "report_type": "BUG",
        "reporter": format!("codelaw {}", codelaw_core::VERSION),
        "result": if c.errors > 0 { "FAILED" } else { "PASSED" },
        "data": [
            { "title": "Errors", "type": "NUMBER", "value": c.errors },
            { "title": "Warnings", "type": "NUMBER", "value": c.warnings },
            { "title": "Infos", "type": "NUMBER", "value": c.infos },
            { "title": "Auto-fixable", "type": "NUMBER", "value": c.fixable },
            { "title": "Files checked", "type": "NUMBER", "value": report.files_checked },
        ],
    });
    let prints = fingerprints(&report.diagnostics);
    let annotations: Vec<Value> = order[..shown]
        .iter()
        .map(|&i| {
            let d = &report.diagnostics[i];
            let (kind, severity) = match d.severity {
                Severity::Error => ("BUG", "HIGH"),
                Severity::Warn => ("CODE_SMELL", "MEDIUM"),
                Severity::Info => ("CODE_SMELL", "LOW"),
            };
            json!({
                "external_id": format!("codelaw-{}", prints[i]),
                "annotation_type": kind,
                "summary": clip(&format!("{}: {} {}", d.rule, d.message, fix_tail(d)), BITBUCKET_SUMMARY),
                "details": clip(&format!("{}:{}:{} {}\n{}", d.file, d.start.line, d.start.column, d.message, fix_lines(d)), BITBUCKET_DETAILS),
                "path": d.file,
                "line": d.start.line.max(1),
                "severity": severity,
                "result": "FAILED",
            })
        })
        .collect();
    let mut out = serde_json::to_string(&head).expect("json") + "\n";
    for batch in annotations.chunks(BITBUCKET_BATCH) {
        out += &(serde_json::to_string(batch).expect("json") + "\n");
    }
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

    /// Every field against Bitbucket Cloud's published schema (`report`,
    /// `report_annotation`, `report_data` in api.bitbucket.org/swagger.json).
    fn assert_code_insights(lines: &[Value]) {
        let report = lines[0].as_object().unwrap();
        for k in report.keys() {
            assert!(["title", "details", "report_type", "reporter", "result", "data"].contains(&k.as_str()), "report field {k}");
        }
        assert!(["SECURITY", "COVERAGE", "TEST", "BUG"].contains(&report["report_type"].as_str().unwrap()));
        assert!(["PASSED", "FAILED", "PENDING"].contains(&report["result"].as_str().unwrap()));
        let data = report["data"].as_array().unwrap();
        assert!(data.len() <= 10);
        for e in data {
            assert!(["BOOLEAN", "DATE", "DURATION", "LINK", "NUMBER", "PERCENTAGE", "TEXT"].contains(&e["type"].as_str().unwrap()));
            assert!(e["title"].is_string() && e["value"].is_u64());
        }
        let mut ids = std::collections::BTreeSet::new();
        for batch in &lines[1..] {
            let batch = batch.as_array().unwrap();
            assert!(!batch.is_empty() && batch.len() <= BITBUCKET_BATCH);
            for a in batch {
                for k in a.as_object().unwrap().keys() {
                    assert!(["external_id", "annotation_type", "summary", "details", "path", "line", "severity", "result"].contains(&k.as_str()), "annotation field {k}");
                }
                assert!(["VULNERABILITY", "CODE_SMELL", "BUG"].contains(&a["annotation_type"].as_str().unwrap()));
                assert!(["CRITICAL", "HIGH", "MEDIUM", "LOW"].contains(&a["severity"].as_str().unwrap()));
                assert!(["PASSED", "FAILED", "SKIPPED", "IGNORED"].contains(&a["result"].as_str().unwrap()));
                assert!(a["line"].as_u64().unwrap() >= 1);
                assert!(a["summary"].as_str().unwrap().chars().count() <= BITBUCKET_SUMMARY);
                assert!(a["details"].as_str().unwrap().chars().count() <= BITBUCKET_DETAILS);
                assert!(ids.insert(a["external_id"].as_str().unwrap().to_owned()), "external_id repeats");
            }
        }
    }

    fn lines(t: &str) -> Vec<Value> {
        assert!(t.ends_with('\n'));
        t.lines().map(|l| serde_json::from_str(l).unwrap()).collect()
    }

    #[test]
    fn bitbucket_is_a_report_then_batches_of_annotations() {
        let mut w = d("r/w", "src/a.js", 0, &"long ".repeat(200));
        w.severity = Severity::Warn;
        let r = CheckReport { diagnostics: vec![w, d("r/x", "src/b.js", 4, "boom")], files_checked: 3, ..Default::default() };
        let l = lines(&check_bitbucket(&r));
        assert_code_insights(&l);
        assert_eq!(l.len(), 2);
        assert_eq!(l[0]["result"], "FAILED");
        assert_eq!(l[0]["details"], "1 errors, 1 warnings, 0 infos in 3 files checked.");
        let a = l[1].as_array().unwrap();
        assert_eq!(a[0]["path"], "src/b.js", "errors first");
        assert_eq!((a[0]["line"].as_u64(), a[0]["severity"].as_str(), a[0]["annotation_type"].as_str()), (Some(4), Some("HIGH"), Some("BUG")));
        assert_eq!(a[0]["summary"], "r/x: boom [fix: manual — Do it.]");
        assert_eq!((a[1]["line"].as_u64(), a[1]["severity"].as_str()), (Some(1), Some("MEDIUM")), "line 0 is sent as 1");
        assert!(a[1]["summary"].as_str().unwrap().ends_with('…'));
        let clean = lines(&check_bitbucket(&CheckReport { files_checked: 1, ..Default::default() }));
        assert_eq!(clean.len(), 1, "no findings, no annotation batch");
        assert_eq!(clean[0]["result"], "PASSED");
    }

    #[test]
    fn bitbucket_truncates_at_the_cap_and_says_so() {
        let mut ds: Vec<Diagnostic> = (0..1203).map(|i| {
            let mut x = d("r/w", "a.js", i + 1, "m");
            x.severity = Severity::Warn;
            x
        }).collect();
        ds.push(d("r/x", "z.js", 1, "the error"));
        let l = lines(&check_bitbucket(&CheckReport { diagnostics: ds, files_checked: 2, ..Default::default() }));
        assert_code_insights(&l);
        assert_eq!(l.len(), 11, "the report and ten batches of 100");
        assert_eq!(l[1][0]["summary"], "r/x: the error [fix: manual — Do it.]", "the error survives the cut");
        assert!(l[0]["details"].as_str().unwrap().contains("Showing 1000 of 1204 findings"), "{}", l[0]["details"]);
    }
}
