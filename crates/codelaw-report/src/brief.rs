//! `codelaw brief --for <path>…` and MCP `brief_for_task` (SR-CLI-MCP
//! decision 14): only what is specific to this repository for the paths an
//! agent is about to write — its own rules, the pack rules it configures,
//! the rules with exceptions there, the `manual` security rules — and one
//! line counting the standard rest. An agent already knows ESLint's and
//! Ruff's correctness rules; it cannot know this repo's choices.
//!
//! Both surfaces call [`brief`], so the bytes are the same (L5). The order
//! is fixed and nothing is ranked: the same files and config, the same list.

use crate::rules_json;
use codelaw_core::FixClass;
use codelaw_engine::{Project, RuleRow};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The brief for a set of paths.
#[derive(Clone, Debug, Default)]
pub struct Brief {
    /// The paths, as given.
    pub paths: Vec<String>,
    /// The repo's own rules (`.codelaw/rules/`).
    pub own: Vec<RuleRow>,
    /// Pack rules whose options a config layer for one of the paths sets.
    pub configured: Vec<(RuleRow, BTreeMap<String, Value>)>,
    /// Rules with exceptions at or under the paths: each kind's count, and
    /// the uses they cover.
    pub exceptions: Vec<(RuleRow, BTreeMap<&'static str, usize>, usize)>,
    /// Security rules whose fix is `manual`.
    pub security: Vec<RuleRow>,
    /// How many applying rules are left, and the tools they are ported from.
    pub standard: usize,
    pub ported_from: Vec<String>,
}

/// A path as the config sees it: absolute, through the real folders, even
/// when the file does not exist yet.
fn absolute(path: &Path) -> PathBuf {
    if let Ok(p) = std::fs::canonicalize(path) {
        return p;
    }
    let abs = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir().map(|d| d.join(path)).unwrap_or_else(|_| path.to_path_buf()) };
    match (abs.parent().and_then(|d| std::fs::canonicalize(d).ok()), abs.file_name()) {
        (Some(dir), Some(name)) => dir.join(name),
        _ => abs,
    }
}

/// Build the brief for `paths`, which need not exist; each one's extension
/// picks its language.
pub fn brief(project: &mut Project, paths: &[PathBuf]) -> Result<Brief, String> {
    if paths.is_empty() {
        return Err("`brief` needs at least one path: `--for <path>`".into());
    }
    // Every rule that applies to one of the paths, once, by id.
    let mut applying: BTreeMap<String, RuleRow> = BTreeMap::new();
    let mut options: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    let mut rels: Vec<String> = Vec::new();
    for path in paths {
        let rows = project.rules_for(path).map_err(|e| e.to_string())?;
        let abs = absolute(path);
        let rel = codelaw_core::normalize_path(abs.strip_prefix(project.root()).unwrap_or(&abs));
        let layers = project.config.layers_for(&abs);
        for row in rows {
            // The options the config layers set for this file, later
            // layers winning, as the engine merges them.
            for layer in &layers {
                if let Some(o) = layer.rules.get(&row.id).and_then(|s| s.options()).filter(|o| !o.is_empty()) {
                    options.entry(row.id.clone()).or_default().extend(o.iter().map(|(k, v)| (k.clone(), v.clone())));
                }
            }
            applying.entry(row.id.clone()).or_insert(row);
        }
        rels.push(rel);
    }
    let all = project.exceptions().map_err(|e| e.to_string())?;
    let under = |file: &str| rels.iter().any(|r| file == r || file.strip_prefix(r.as_str()).is_some_and(|rest| rest.starts_with('/')) || r == ".");
    let mut excepted: BTreeMap<String, (BTreeMap<&'static str, usize>, usize)> = BTreeMap::new();
    for e in all.items.iter().filter(|e| e.files.iter().any(|f| under(f))) {
        let entry = excepted.entry(e.rule.clone()).or_default();
        *entry.0.entry(e.kind.name()).or_default() += 1;
        entry.1 += e.uses;
    }
    let ported: BTreeMap<&str, &[String]> = project.compiled.iter().map(|c| (c.loaded.rule.id.as_str(), c.loaded.rule.ported_from.as_slice())).collect();

    let mut out = Brief { paths: paths.iter().map(|p| codelaw_core::normalize_path(p)).collect(), ..Brief::default() };
    let mut tools: Vec<String> = Vec::new();
    for (id, row) in applying {
        if row.source == "local" {
            out.own.push(row);
        } else if let Some(o) = options.remove(&id) {
            out.configured.push((row, o));
        } else if let Some((kinds, uses)) = excepted.remove(&id) {
            out.exceptions.push((row, kinds, uses));
        } else if row.security.is_some() && row.fix == FixClass::Manual {
            out.security.push(row);
        } else {
            out.standard += 1;
            for p in ported.get(id.as_str()).copied().unwrap_or_default() {
                let tool = p.split('/').next().unwrap_or(p).to_owned();
                if !tool.is_empty() && !tools.contains(&tool) {
                    tools.push(tool);
                }
            }
        }
    }
    tools.sort();
    out.ported_from = tools;
    Ok(out)
}

/// One rule's card, as `rules --for` gives it.
fn card(r: &RuleRow) -> Value {
    rules_json(std::slice::from_ref(r), None)["rules"][0].clone()
}

fn with(mut card: Value, key: &str, value: Value) -> Value {
    card[key] = value;
    card
}

/// `brief --format json` and `brief_for_task`.
pub fn brief_json(b: &Brief) -> Value {
    let mut out = Map::new();
    out.insert("schema_version".into(), crate::SCHEMA_VERSION.into());
    out.insert("codelaw_version".into(), codelaw_core::VERSION.into());
    out.insert("for".into(), json!(b.paths));
    out.insert("own".into(), b.own.iter().map(card).collect());
    out.insert("configured".into(), b.configured.iter().map(|(r, o)| with(card(r), "options", json!(o))).collect());
    out.insert("exceptions".into(), b.exceptions.iter().map(|(r, k, u)| with(with(card(r), "kinds", json!(k)), "uses", json!(u))).collect());
    out.insert("security".into(), b.security.iter().map(card).collect());
    out.insert("standard".into(), json!({ "count": b.standard, "ported_from": b.ported_from }));
    Value::Object(out)
}

/// The closing line: how many standard rules are left.
fn closing(b: &Brief) -> String {
    let from = if b.ported_from.is_empty() { String::new() } else { format!(" ({})", b.ported_from.join(", ")) };
    let s = if b.standard == 1 { "" } else { "s" };
    let other = if b.own.len() + b.configured.len() + b.exceptions.len() + b.security.len() == 0 { "the" } else { "the other" };
    format!("{other} {} rule{s} {} standard{from}; `codelaw hook --pre` refuses a write that breaks one, with its fix\n", b.standard, if b.standard == 1 { "is" } else { "are" })
}

fn line(r: &RuleRow, extra: &str) -> String {
    format!("  {:<40} {:<6} {:<8} {}{extra}\n", r.id, r.severity, r.fix.name(), r.title)
}

/// `brief` as text: a heading per group that has rules, then the closing
/// line.
pub fn brief_text(b: &Brief) -> String {
    let mut out = format!("codelaw brief for {}\n", b.paths.join(", "));
    if !b.own.is_empty() {
        out.push_str("\nthis repo's own rules:\n");
        for r in &b.own {
            out.push_str(&line(r, ""));
        }
    }
    if !b.configured.is_empty() {
        out.push_str("\nrules this repo configures:\n");
        for (r, o) in &b.configured {
            let set: Vec<String> = o.iter().map(|(k, v)| format!("{k} = {v}")).collect();
            out.push_str(&line(r, &format!("  ({})", set.join(", "))));
        }
    }
    if !b.exceptions.is_empty() {
        out.push_str("\nrules with exceptions here (what this code has got wrong):\n");
        for (r, k, u) in &b.exceptions {
            let kinds: Vec<String> = k.iter().map(|(kind, n)| format!("{n} {kind}")).collect();
            out.push_str(&line(r, &format!("  ({}; {u} uses)", kinds.join(", "))));
        }
    }
    if !b.security.is_empty() {
        out.push_str("\nsecurity rules with a manual fix:\n");
        for r in &b.security {
            let cwe = r.security.as_ref().map(|s| s.cwe_label()).filter(|l| !l.is_empty()).map(|l| format!("  {l}")).unwrap_or_default();
            out.push_str(&line(r, &cwe));
        }
    }
    out.push('\n');
    out.push_str(&closing(b));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str) -> RuleRow {
        RuleRow {
            id: id.into(),
            title: "t".into(),
            severity: "error".into(),
            fix: FixClass::Auto,
            source: "p@1".into(),
            preview: false,
            tags: Vec::new(),
            security: None,
            ported_from: Vec::new(),
        }
    }

    #[test]
    fn the_closing_line_counts_what_is_left() {
        let b = Brief { paths: vec!["a.ts".into()], standard: 3, ported_from: vec!["eslint".into(), "ruff".into()], ..Brief::default() };
        assert_eq!(closing(&b), "the 3 rules are standard (eslint, ruff); `codelaw hook --pre` refuses a write that breaks one, with its fix\n");
        let b = Brief { own: vec![row("local/x")], standard: 1, ..b };
        assert!(closing(&b).starts_with("the other 1 rule is standard"), "{}", closing(&b));
        let v = brief_json(&b);
        assert_eq!(v["own"][0]["id"], "local/x");
        assert_eq!(v["standard"]["count"], 1);
    }
}
