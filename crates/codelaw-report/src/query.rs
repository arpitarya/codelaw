//! The text and `--format json` shapes of `rules`, `rules --why`,
//! `explain` and `tree` — the same objects the MCP tools `list_rules`,
//! `rules_for_path`, `trace_rule`, `explain_rule` and `dump_tree` return
//! (SR-CLI-MCP decisions 5 and 6).

use crate::{SCHEMA_VERSION, diagnostic_json};
use codelaw_core::{FixClass, Language, LineIndex};
use codelaw_engine::{FileFix, RuleRow, RuleWhy, severity_name};
use codelaw_rules::{EngineRule, LoadedRule, OWASP_TOP_10_2025, Security};
use serde_json::{Map, Value, json};

fn envelope(body: Value) -> Value {
    let mut out = Map::new();
    out.insert("schema_version".into(), SCHEMA_VERSION.into());
    out.insert("codelaw_version".into(), codelaw_core::VERSION.into());
    if let Value::Object(fields) = body {
        out.extend(fields);
    }
    Value::Object(out)
}

/// A security rule's `security` object in JSON.
pub fn security_json(s: &Security) -> Value {
    json!({ "cwe": s.cwe, "owasp": s.owasp, "severity": s.severity.name(), "security_severity": s.severity.score() })
}

/// `rules [--for <file>] --format json`: one card per rule; a security
/// rule's card adds `security` (SR-RULE-FORMAT decision 2a).
pub fn rules_json(rows: &[RuleRow], for_file: Option<&str>) -> Value {
    let rules: Vec<Value> = rows
        .iter()
        .map(|r| {
            let mut card = json!({ "id": r.id, "severity": r.severity, "fix": r.fix.name(), "title": r.title, "source": r.source, "preview": r.preview });
            if let Some(s) = &r.security {
                card["security"] = security_json(s);
            }
            if !r.ported_from.is_empty() {
                card["ported_from"] = json!(r.ported_from);
            }
            card
        })
        .collect();
    envelope(json!({ "for": for_file, "rules": rules }))
}

/// `rules_json` with the packages the file brings in that are vulnerable
/// or a major version behind (`Project::dependency_notes`, W-128): under
/// `dependencies`, only when there are some, so every other answer keeps
/// its bytes.
pub fn with_dependencies(mut v: Value, notes: Vec<Value>) -> Value {
    if !notes.is_empty() {
        v["dependencies"] = Value::Array(notes);
    }
    v
}

/// `rules [--for <file>]` as text: one line per rule; a security rule's
/// line names its CWE ids.
pub fn rules_text(rows: &[RuleRow]) -> String {
    let mut out = String::new();
    for r in rows {
        let preview = if r.preview { "  (preview)" } else { "" };
        let cwe = r.security.as_ref().map(|s| s.cwe_label()).filter(|l| !l.is_empty()).map(|l| format!("  {l}")).unwrap_or_default();
        out.push_str(&format!("{:<40} {:<6} {:<8} {}{preview}{cwe}\n", r.id, r.severity, r.fix.name(), r.source));
    }
    out
}

/// `rules --security-coverage`: each OWASP Top 10:2025 category and the
/// security rules mapped to it, then the security rules mapped to none.
/// An OWASP category's id and name, and the rules mapped to it.
type Category<'a> = (&'static str, &'static str, Vec<&'a RuleRow>);

fn coverage(rows: &[RuleRow]) -> (Vec<Category<'_>>, Vec<&RuleRow>) {
    let security: Vec<&RuleRow> = rows.iter().filter(|r| r.security.is_some()).collect();
    let categories = OWASP_TOP_10_2025
        .iter()
        .map(|(id, name)| {
            let hits = security.iter().copied().filter(|r| r.security.as_ref().is_some_and(|s| s.owasp.iter().any(|o| o == id))).collect();
            (*id, *name, hits)
        })
        .collect();
    let unmapped =
        security.iter().copied().filter(|r| r.security.as_ref().is_some_and(|s| !s.owasp.iter().any(|o| OWASP_TOP_10_2025.iter().any(|(id, _)| id == o)))).collect();
    (categories, unmapped)
}

pub fn security_coverage_text(rows: &[RuleRow]) -> String {
    let (categories, unmapped) = coverage(rows);
    let mut out = String::from("OWASP Top 10:2025 × the rules this project loads\n\n");
    for (id, name, hits) in &categories {
        let list = if hits.is_empty() {
            "—".to_owned()
        } else {
            hits.iter().map(|r| format!("{} {}", r.id, r.security.as_ref().map(|s| s.cwe_label()).unwrap_or_default()).trim_end().to_owned()).collect::<Vec<_>>().join(", ")
        };
        out.push_str(&format!("{id}  {name:<40} {:>3}  {list}\n", hits.len()));
    }
    let covered = categories.iter().filter(|(_, _, h)| !h.is_empty()).count();
    let total = rows.iter().filter(|r| r.security.is_some()).count();
    out.push_str(&format!("\n{covered} of 10 categories covered by {total} security rule(s)"));
    if !unmapped.is_empty() {
        out.push_str(&format!("; {} mapped to no 2025 category: {}", unmapped.len(), unmapped.iter().map(|r| r.id.as_str()).collect::<Vec<_>>().join(", ")));
    }
    out.push('\n');
    out
}

pub fn security_coverage_json(rows: &[RuleRow]) -> Value {
    let (categories, unmapped) = coverage(rows);
    let card = |r: &&RuleRow| json!({ "id": r.id, "severity": r.severity, "source": r.source, "security": r.security.as_ref().map(security_json) });
    envelope(json!({
        "owasp": categories.iter().map(|(id, name, hits)| json!({ "id": id, "name": name, "rules": hits.iter().map(card).collect::<Vec<_>>() })).collect::<Vec<_>>(),
        "unmapped": unmapped.iter().map(card).collect::<Vec<_>>(),
    }))
}

/// `rules --why <id> [--for <file>] --format json`.
pub fn why_json(w: &RuleWhy) -> Value {
    let for_file = w.for_file.as_ref().map(|f| {
        json!({
            "file": f.file,
            "set_by": f.set_by.iter().map(|(source, value)| json!({ "source": source, "value": value })).collect::<Vec<_>>(),
            "applies": f.applies,
            "severity": f.severity,
            "diagnostics": f.diagnostics.iter().map(diagnostic_json).collect::<Vec<_>>(),
            "match": f.matches.as_ref().map(crate::debug::match_nodes_json),
        })
    });
    envelope(json!({
        "id": w.id,
        "title": w.title,
        "source": w.source,
        "path": w.path,
        "shadows": w.shadows,
        "enabled_by": w.enabled_by,
        "default": w.default,
        "preview": w.preview,
        "for": for_file,
    }))
}

/// `rules --why <id> [--for <file>]` as text.
pub fn why_text(w: &RuleWhy) -> String {
    let mut out = format!("{}  {}\n", w.id, w.title);
    out.push_str(&format!("  source:     {} ({})\n", w.source, w.path));
    if !w.shadows.is_empty() {
        out.push_str(&format!("  shadows:    {}\n", w.shadows.join(", ")));
    }
    out.push_str(&format!("  enabled by: {}\n", w.enabled_by.as_deref().unwrap_or("nothing (not in any extended preset)")));
    out.push_str(&format!("  default:    {}\n", w.default));
    if w.preview {
        out.push_str("  preview:    yes — runs only with `preview = true`\n");
    }
    if let Some(f) = &w.for_file {
        for (source, value) in &f.set_by {
            out.push_str(&format!("  set by:     {source} = {value}\n"));
        }
        out.push_str(&format!("  for {}:  {}\n", f.file, if f.applies { f.severity.clone() } else { "does not apply (language or `files`)".into() }));
        for d in &f.diagnostics {
            out.push_str(&format!("  reports:    {}:{} {}\n", d.start.line, d.start.column, d.message));
        }
        if let Some(m) = &f.matches {
            let matched = m.nodes.iter().filter(|(_, _, n)| n.matched).count();
            out.push_str(&format!(
                "  match:      {matched} matches among {} candidate nodes; `codelaw match {} {} --explain` shows each clause\n",
                m.nodes.len(),
                w.id,
                m.file
            ));
        }
    }
    out
}

fn snippets_json(snippets: &[codelaw_rules::Snippet]) -> Vec<Value> {
    snippets.iter().map(|s| json!({ "code": s.code, "path": s.path })).collect()
}

/// `explain <id> --format json` for a rule of the project.
pub fn explain_json(r: &LoadedRule) -> Value {
    let rule = &r.rule;
    let examples: Vec<Value> = rule
        .examples
        .iter()
        .map(|e| json!({ "language": e.language.name(), "dont": snippets_json(&e.donts), "do": snippets_json(&e.dos), "fixed": snippets_json(&e.fixed) }))
        .collect();
    envelope(json!({
        "id": rule.id,
        "title": rule.title,
        "summary": rule.summary,
        "why": rule.why,
        "languages": rule.languages.iter().map(|l| l.name()).collect::<Vec<_>>(),
        "severity": severity_name(&r.default),
        "fix": { "class": rule.fix.name(), "reason": rule.fix_reason, "skill": fix_skill(r), "steps": rule.steps },
        "ported_from": rule.ported_from,
        "source": r.source.label(),
        "path": codelaw_core::normalize_path(&rule.path),
        "examples": examples,
    }))
}

/// The skill that carries a rule's `manual` fix: the one it names, else
/// `codelaw-fix` (L4 decision 10).
pub fn fix_skill(r: &LoadedRule) -> Option<String> {
    match (&r.rule.fix_skill, r.rule.fix) {
        (Some(s), _) => Some(s.clone()),
        (None, FixClass::Manual) => Some("codelaw-fix".into()),
        _ => None,
    }
}

/// `explain <id> --format json` for one of the engine's own rules.
pub fn explain_engine_json(e: &EngineRule) -> Value {
    envelope(json!({
        "id": e.id,
        "title": e.title,
        "summary": "",
        "why": "",
        "languages": [],
        "severity": e.severity.name(),
        "fix": { "class": e.fix.name(), "reason": null, "skill": if e.fix == FixClass::Manual { Some("codelaw-fix") } else { None }, "steps": e.steps },
        "ported_from": [],
        "source": "engine",
        "path": null,
        "examples": [],
    }))
}

/// `explain <id>` as text for a rule of the project.
pub fn explain_text(r: &LoadedRule) -> String {
    let rule = &r.rule;
    let langs: Vec<&str> = rule.languages.iter().map(|l| l.name()).collect();
    let mut out = format!("{}  {}\n\n", rule.id, rule.title);
    out.push_str(&format!("{}\n\n", rule.summary));
    out.push_str(&format!("Why: {}\n\n", rule.why));
    out.push_str(&format!(
        "  languages: {}  ·  severity: {}  ·  fix: {}  ·  source: {}\n",
        langs.join(", "),
        severity_name(&r.default),
        rule.fix,
        r.source.label()
    ));
    if let Some(reason) = &rule.fix_reason {
        out.push_str(&format!("  not automatic because: {reason}\n"));
    }
    if !rule.ported_from.is_empty() {
        out.push_str(&format!("  ported from: {}\n", rule.ported_from.join(", ")));
    }
    if rule.fix == FixClass::Manual || !rule.steps.is_empty() {
        out.push_str("\nHow to fix:\n");
        for (i, s) in rule.steps.iter().enumerate() {
            out.push_str(&format!("  {}. {s}\n", i + 1));
        }
    }
    out.push_str(&format!("\n{}\n", codelaw_core::normalize_path(&rule.path)));
    out
}

/// `explain <id>` as text for one of the engine's own rules.
pub fn explain_engine_text(e: &EngineRule) -> String {
    let mut out = format!("{}  {}\n\n  severity: {}  ·  fix: {}  ·  source: engine\n", e.id, e.title, e.severity, e.fix);
    for (i, s) in e.steps.iter().enumerate() {
        out.push_str(&format!("  {}. {s}\n", i + 1));
    }
    out
}

/// MCP `format_snippet`: a snippet after `fix` (its `auto` fixes, and the
/// format step once `fix` has one), with what is left. The text is the
/// bytes `codelaw fix` would write to that file.
pub fn snippet_fix_json(f: &FileFix) -> Value {
    envelope(json!({
        "file": f.file,
        "code": f.fixed,
        "changed": f.changed(),
        "applied": f.applied,
        "passes": f.passes,
        "diagnostics": f.remaining.iter().map(diagnostic_json).collect::<Vec<_>>(),
    }))
}

fn on_line(node: tree_sitter::Node<'_>, index: &LineIndex<'_>, line: Option<u32>) -> bool {
    line.is_none_or(|l| index.position(node.start_byte()).line <= l && l <= index.position(node.end_byte()).line)
}

fn point(index: &LineIndex<'_>, offset: usize) -> Value {
    let p = index.position(offset);
    json!({ "line": p.line, "column": p.column, "offset": offset })
}

/// Named nodes under `node` (anonymous ones are walked through), as JSON.
fn tree_nodes(node: tree_sitter::Node<'_>, field: Option<&str>, text: &str, index: &LineIndex<'_>, line: Option<u32>, out: &mut Vec<Value>) {
    if !on_line(node, index, line) {
        return;
    }
    let mut children = Vec::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            tree_nodes(cursor.node(), cursor.field_name(), text, index, line, &mut children);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    if node.is_named() || node.is_missing() {
        let mut n = Map::new();
        n.insert("kind".into(), node.kind().into());
        n.insert("field".into(), field.into());
        n.insert("range".into(), json!({ "start": point(index, node.start_byte()), "end": point(index, node.end_byte()) }));
        if node.is_error() {
            n.insert("error".into(), true.into());
        }
        if node.is_missing() {
            n.insert("missing".into(), true.into());
        }
        if node.named_child_count() == 0 {
            n.insert("text".into(), text[node.byte_range()].into());
        }
        n.insert("children".into(), children.into());
        out.push(Value::Object(n));
    } else {
        out.extend(children);
    }
}

/// `tree --format json`: the named syntax tree the engine sees.
pub fn tree_json(language: Language, text: &str, line: Option<u32>) -> Value {
    let root = codelaw_lang::parse(language, text);
    let index = LineIndex::new(text);
    let mut nodes = Vec::new();
    tree_nodes(root.root().get_inner_node(), None, text, &index, line, &mut nodes);
    envelope(json!({ "language": language.name(), "line": line, "nodes": nodes }))
}

fn tree_lines(node: tree_sitter::Node<'_>, field: Option<&str>, depth: usize, text: &str, index: &LineIndex<'_>, line: Option<u32>, out: &mut String) {
    if !on_line(node, index, line) {
        return;
    }
    if node.is_named() || node.is_missing() {
        let start = index.position(node.start_byte());
        let end = index.position(node.end_byte());
        let label = field.map(|f| format!("{f}: ")).unwrap_or_default();
        let snippet = if node.named_child_count() == 0 {
            let t: String = text[node.byte_range()].chars().take(40).collect();
            format!("  {t:?}")
        } else {
            String::new()
        };
        let flag = if node.is_error() {
            " ERROR"
        } else if node.is_missing() {
            " MISSING"
        } else {
            ""
        };
        out.push_str(&format!("{}{label}{}{flag}  [{}:{}-{}:{}]{snippet}\n", "  ".repeat(depth), node.kind(), start.line, start.column, end.line, end.column));
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let next_depth = if node.is_named() { depth + 1 } else { depth };
            tree_lines(cursor.node(), cursor.field_name(), next_depth, text, index, line, out);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

/// `tree` as text: one indented line per named node.
pub fn tree_text(language: Language, text: &str, line: Option<u32>) -> String {
    let root = codelaw_lang::parse(language, text);
    let index = LineIndex::new(text);
    let mut out = String::new();
    tree_lines(root.root().get_inner_node(), None, 0, text, &index, line, &mut out);
    out
}
