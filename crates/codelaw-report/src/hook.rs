//! What `codelaw hook` tells an agent after an edit (SR-AGENT-SURFACES,
//! L4 decision 8): the `auto` fixes it applied, then each finding that is
//! left with what to do next — a `suggest` edit to show as a diff, or the
//! `manual` steps — inside a token budget.

use crate::counts;
use codelaw_core::{Diagnostic, FixClass, Severity};
use codelaw_engine::{CheckReport, FileFix};

/// At most this many findings are spelled out; the rest are counted.
pub const HOOK_FINDINGS: usize = 20;

fn one_line(s: &str, max: usize) -> String {
    let flat = s.replace('\n', "\\n");
    if flat.chars().count() <= max { flat } else { format!("{}…", flat.chars().take(max).collect::<String>()) }
}

fn next_step(d: &Diagnostic) -> String {
    match d.fix {
        FixClass::Auto if !d.edits.is_empty() => "fix: auto — run `codelaw fix` on the file".into(),
        FixClass::Suggest if !d.edits.is_empty() => {
            let edits: Vec<String> = d.edits.iter().map(|e| format!("bytes {}..{} → \"{}\"", e.start, e.end, one_line(&e.text, 60))).collect();
            format!("fix: suggest — show the user this diff and apply it only once accepted: {}", edits.join("; "))
        }
        _ if d.steps.is_empty() => "fix: manual — see `codelaw explain` for the steps".into(),
        _ => format!("fix: manual — {}", d.steps.iter().enumerate().map(|(i, s)| format!("{}. {s}", i + 1)).collect::<Vec<_>>().join(" ")),
    }
}

/// Whether what is left must go back to the agent: an error, a warning,
/// or a rule that failed.
pub fn hook_blocks(report: &CheckReport) -> bool {
    let c = counts(&report.diagnostics);
    c.errors + c.warnings > 0 || report.internal_errors() > 0
}

/// The hook's message: fixes applied, then what is left, worst first.
pub fn hook_text(fixed: &[FileFix], report: &CheckReport) -> String {
    let mut out = String::new();
    for f in fixed.iter().filter(|f| f.written) {
        let applied: Vec<String> = f.applied.iter().map(|(r, n)| format!("{r} ×{n}")).collect();
        out.push_str(&format!("codelaw auto-fixed {}: {}\n", f.file, applied.join(", ")));
    }
    let mut left: Vec<&Diagnostic> = report.diagnostics.iter().filter(|d| d.severity != Severity::Info || d.rule == "codelaw/internal-error").collect();
    left.sort_by_key(|d| match d.severity {
        Severity::Error => 0,
        Severity::Warn => 1,
        Severity::Info => 2,
    });
    for d in left.iter().take(HOOK_FINDINGS) {
        out.push_str(&format!("{}:{}:{} {} {} {} — {}\n", d.file, d.start.line, d.start.column, d.severity.name(), d.rule, d.message, next_step(d)));
    }
    if left.len() > HOOK_FINDINGS {
        out.push_str(&format!("… and {} more; run `codelaw check --format json` on these files for all of them\n", left.len() - HOOK_FINDINGS));
    }
    for s in &report.skipped {
        out.push_str(&format!("{} not checked: {}\n", s.file, s.reason));
    }
    if hook_blocks(report) {
        let c = counts(&report.diagnostics);
        out.push_str(&format!(
            "codelaw: {} errors, {} warnings left. Fix them now (the codelaw-fix skill); never suppress or disable a rule to pass.\n",
            c.errors, c.warnings
        ));
    }
    out
}
