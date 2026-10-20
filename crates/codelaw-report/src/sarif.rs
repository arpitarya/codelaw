//! `--format sarif`: SARIF 2.1.0 for code scanning (SR-CLI-MCP decision 4).
//! It carries the L4 fix fields: `auto` and `suggest` edits as SARIF
//! `fixes`, and the class, steps and note in each result's `properties`.

use codelaw_core::{Diagnostic, FixClass, Severity};
use codelaw_engine::{CheckReport, Project};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The schema SARIF files name; tests validate against it.
pub const SARIF_SCHEMA: &str = "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";

fn level(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warn => "warning",
        Severity::Info => "note",
    }
}

fn artifact(file: &str) -> Value {
    json!({ "uri": file })
}

fn result(d: &Diagnostic, rule_index: Option<usize>) -> Value {
    let mut region = json!({
        "startLine": d.start.line,
        "startColumn": d.start.column,
        "endLine": d.end.line,
        "endColumn": d.end.column,
        "byteOffset": d.start_byte,
        "byteLength": d.end_byte.saturating_sub(d.start_byte),
    });
    // A whole-file diagnostic (a skipped file, say) has no span.
    if d.start_byte == d.end_byte && d.start.line == 1 && d.start.column == 1 {
        region = json!({ "startLine": 1 });
    }
    let mut r = json!({
        "ruleId": d.rule,
        "level": level(d.severity),
        "message": { "text": d.message },
        "locations": [{ "physicalLocation": { "artifactLocation": artifact(&d.file), "region": region } }],
        "properties": {
            "fixClass": d.fix.name(),
            "steps": d.steps,
            "note": d.note,
            "source": d.source,
        },
    });
    if let Some(i) = rule_index {
        r["ruleIndex"] = json!(i);
    }
    if d.fix != FixClass::Manual && !d.edits.is_empty() {
        let replacements: Vec<Value> = d
            .edits
            .iter()
            .map(|e| json!({ "deletedRegion": { "byteOffset": e.start, "byteLength": e.end - e.start }, "insertedContent": { "text": e.text } }))
            .collect();
        r["fixes"] = json!([{
            "description": { "text": format!("{} fix for {}", d.fix.name(), d.rule) },
            "artifactChanges": [{ "artifactLocation": artifact(&d.file), "replacements": replacements }],
        }]);
    }
    r
}

/// The SARIF log for a check (or for what a fix left).
pub fn check_sarif(report: &CheckReport, project: &Project) -> Value {
    // Every rule a result names, in id order: the project's rules first,
    // then the engine's own (`codelaw/…`).
    let mut rules: BTreeMap<&str, Value> = BTreeMap::new();
    for d in &report.diagnostics {
        if rules.contains_key(d.rule.as_str()) {
            continue;
        }
        let meta = if let Some(r) = project.rules.get(&d.rule) {
            json!({
                "id": r.rule.id,
                "name": r.rule.id,
                "shortDescription": { "text": r.rule.title },
                "fullDescription": { "text": r.rule.summary },
                "help": { "text": format!("{}\n\nWhy: {}\n\n`codelaw explain {}`", r.rule.summary, r.rule.why, r.rule.id) },
                "properties": { "fixClass": r.rule.fix.name(), "source": r.source.label(), "languages": r.rule.languages.iter().map(|l| l.name()).collect::<Vec<_>>() },
            })
        } else if let Some(e) = codelaw_rules::engine_rule(&d.rule) {
            json!({
                "id": e.id,
                "name": e.id,
                "shortDescription": { "text": e.title },
                "defaultConfiguration": { "level": level(e.severity) },
                "properties": { "fixClass": e.fix.name(), "source": "engine" },
            })
        } else {
            json!({ "id": d.rule })
        };
        rules.insert(d.rule.as_str(), meta);
    }
    let index: BTreeMap<&str, usize> = rules.keys().enumerate().map(|(i, k)| (*k, i)).collect();
    let mut notifications: Vec<Value> = report.warnings.iter().map(|w| json!({ "level": "warning", "message": { "text": w } })).collect();
    for s in &report.skipped {
        notifications.push(json!({
            "level": "note",
            "message": { "text": format!("skipped: {}", s.reason) },
            "locations": [{ "physicalLocation": { "artifactLocation": artifact(&s.file) } }],
        }));
    }
    json!({
        "$schema": SARIF_SCHEMA,
        "version": "2.1.0",
        "runs": [{
            "tool": { "driver": {
                "name": "codelaw",
                "version": codelaw_core::VERSION,
                "semanticVersion": codelaw_core::VERSION,
                "informationUri": "https://github.com/arpitarya/codelaw",
                "rules": rules.into_values().collect::<Vec<_>>(),
            } },
            "columnKind": "unicodeCodePoints",
            "invocations": [{
                "executionSuccessful": report.internal_errors() == 0,
                "toolExecutionNotifications": notifications,
            }],
            "results": report.diagnostics.iter().map(|d| result(d, index.get(d.rule.as_str()).copied())).collect::<Vec<_>>(),
            "properties": { "filesChecked": report.files_checked, "filesSkipped": report.skipped.len() },
        }],
    })
}
