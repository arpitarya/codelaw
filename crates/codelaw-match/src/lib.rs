//! `match:` rules: an ast-grep rule (patterns, kinds and the relational
//! clauses `inside`, `has`, `precedes`, `follows`) plus codelaw's semantic
//! predicates under `where:` (SR-ARCHITECTURE decision 15a).
//!
//! ```yaml
//! match:
//!   kind: identifier
//!   inside: { kind: variable_declarator, field: name }
//! where:
//!   $$MATCH: unused                    # the matched node itself
//!   $NAME: [reassigned, { not: shadowed }]
//!   $CALLEE: { imported-from: { module: react, name: useState } }
//!   $REF: { undeclared: { globals: [window], tables: [globals/browser] } }
//! ```

pub mod data;
mod predicate;
pub mod schema;
pub mod target;

pub use predicate::{Predicate, parse_predicates};
pub use target::{CompiledTarget, Pred, Slot, TargetFinding, compile_target};

use ast_grep_config::{DeserializeEnv, RuleCore, SerializableRuleCore};
use ast_grep_core::Matcher;
use ast_grep_core::matcher::MatcherExt;
use codelaw_lang::{Grammar, SyntaxNode};
use codelaw_rules::{BlockKind, RuleFile};
use codelaw_semantic::SemanticModel;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The capture key for the matched node itself.
pub const MATCH_KEY: &str = "$$MATCH";

/// A captured node, owned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Captured {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub kind: String,
}

/// One match of a rule on one file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub kind: String,
    pub single: BTreeMap<String, Captured>,
    pub multi: BTreeMap<String, Vec<Captured>>,
}

impl Match {
    /// The source span a multi-capture covers, or `None` if it is empty.
    pub fn multi_span(&self, name: &str) -> Option<(usize, usize)> {
        let m = self.multi.get(name)?;
        Some((m.first()?.start, m.last()?.end))
    }
}

/// A `match:` rule compiled for one grammar.
pub struct CompiledMatch {
    core: RuleCore,
    predicates: Vec<(String, Predicate)>,
}

fn captured(n: &SyntaxNode<'_>) -> Captured {
    let r = n.range();
    Captured { start: r.start, end: r.end, text: n.text().to_string(), kind: n.kind().to_string() }
}

/// Every node kind a `match:` block names, in its rule, constraints and
/// utils.
pub fn kinds_named(rule: &RuleFile) -> Vec<String> {
    fn walk(v: &serde_json::Value, out: &mut Vec<String>) {
        match v {
            serde_json::Value::Object(o) => {
                for (k, x) in o {
                    match (k.as_str(), x) {
                        ("kind", serde_json::Value::String(s)) => out.push(s.clone()),
                        _ => walk(x, out),
                    }
                }
            }
            serde_json::Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    if let BlockKind::Match { rule: m, constraints, utils, .. } = &rule.block.kind {
        walk(m, &mut out);
        constraints.iter().chain(utils.iter()).for_each(|v| walk(v, &mut out));
    }
    out.sort();
    out.dedup();
    out
}

/// Whether a grammar has a named node kind.
pub fn grammar_has_kind(grammar: Grammar, kind: &str) -> bool {
    use codelaw_lang::AgLanguage;
    grammar.kind_to_id(kind) != 0
}

/// A rule object naming a kind the grammar doesn't have can never match
/// there: it becomes a rule that matches nothing, so one rule can span
/// languages whose grammars differ (TypeScript's `ambient_declaration` in a
/// rule that also runs on JavaScript).
fn prune_unknown_kinds(v: &mut serde_json::Value, grammar: Grammar, never: &serde_json::Value) {
    match v {
        serde_json::Value::Object(o) => {
            if let Some(serde_json::Value::String(k)) = o.get("kind")
                && !grammar_has_kind(grammar, k)
            {
                *v = never.clone();
                return;
            }
            for x in o.values_mut() {
                prune_unknown_kinds(x, grammar, never);
            }
        }
        serde_json::Value::Array(a) => a.iter_mut().for_each(|x| prune_unknown_kinds(x, grammar, never)),
        _ => {}
    }
}

/// Compile a rule's `match:` block for one grammar. `data` is the rule's
/// pack's tables, for predicates that name one.
pub fn compile(rule: &RuleFile, grammar: Grammar, data: &BTreeMap<String, Arc<serde_json::Value>>) -> Result<CompiledMatch, String> {
    let BlockKind::Match { rule: matcher, constraints, utils, predicates } = &rule.block.kind else {
        return Err("not a `match:` rule".into());
    };
    use ast_grep_core::tree_sitter::LanguageExt;
    let root_kind = grammar.ast_grep("").root().kind().to_string();
    let never = serde_json::json!({ "all": [{ "kind": root_kind }, { "not": { "kind": root_kind } }] });
    let pruned = |v: &serde_json::Value| {
        let mut v = v.clone();
        prune_unknown_kinds(&mut v, grammar, &never);
        v
    };
    let mut core = serde_json::Map::new();
    core.insert("rule".into(), pruned(matcher));
    if let Some(c) = constraints {
        core.insert("constraints".into(), pruned(c));
    }
    if let Some(u) = utils {
        core.insert("utils".into(), pruned(u));
    }
    let serial: SerializableRuleCore =
        serde_json::from_value(serde_json::Value::Object(core)).map_err(|e| format!("invalid `match:`: {e}"))?;
    let core = serial.get_matcher(DeserializeEnv::new(grammar)).map_err(|e| format!("invalid `match:` for {grammar:?}: {e}"))?;
    let predicates = match predicates {
        Some(p) => parse_predicates(p, data)?,
        None => Vec::new(),
    };
    for (var, _) in &predicates {
        if var != MATCH_KEY && !core.defined_vars().contains(var.trim_start_matches('$')) {
            return Err(format!("`where:` names `{var}`, which `match:` does not capture"));
        }
    }
    Ok(CompiledMatch { core, predicates })
}

impl CompiledMatch {
    /// The node kinds this rule can match, or `None` for any (decision 15:
    /// rules are dispatched by kind).
    pub fn kinds(&self) -> Option<Vec<u16>> {
        self.core.potential_kinds().map(|b| b.iter().map(|k| k as u16).collect())
    }

    pub fn needs_semantic(&self) -> bool {
        !self.predicates.is_empty()
    }

    /// Match one node.
    pub fn try_match(&self, node: SyntaxNode<'_>, semantic: Option<&SemanticModel>) -> Option<Match> {
        let m = self.core.match_node(node)?;
        let env = m.get_env();
        let mut single = BTreeMap::new();
        let mut multi = BTreeMap::new();
        for var in self.core.defined_vars() {
            if let Some(n) = env.get_match(var) {
                single.insert(var.to_owned(), captured(n));
            } else {
                // An empty `$$$NAME` is still a capture: it interpolates as "".
                let nodes = env.get_multiple_matches(var);
                multi.insert(var.to_owned(), nodes.iter().map(captured).collect());
            }
        }
        let range = m.range();
        let this = captured(&m);
        for (var, predicate) in &self.predicates {
            let target = if var == MATCH_KEY { Some(&this) } else { single.get(var.trim_start_matches('$')) };
            let holds = match (target, semantic) {
                (Some(t), Some(model)) => predicate.holds(model, t.start),
                _ => false,
            };
            if !holds {
                return None;
            }
        }
        Some(Match { start: range.start, end: range.end, kind: m.kind().to_string(), single, multi })
    }
}

/// Replace `$NAME`, `$$$NAME` and `$$MATCH` (the whole matched text) in a
/// message or template with what the match captured. An unknown name is
/// left as written.
pub fn interpolate(template: &str, m: &Match, source: &str) -> String {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let dollars = bytes[i..].iter().take_while(|b| **b == b'$').count();
            let name_start = i + dollars;
            let name_len = bytes[name_start..].iter().take_while(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || **b == b'_').count();
            if dollars == 2 && &template[name_start..name_start + name_len] == "MATCH" {
                out.push_str(&source[m.start..m.end]);
                i = name_start + name_len;
                continue;
            }
            if name_len > 0 && (dollars == 1 || dollars == 3) {
                let name = &template[name_start..name_start + name_len];
                let value = if dollars == 3 {
                    m.multi_span(name).map(|(s, e)| source[s..e].to_owned()).or_else(|| m.single.get(name).map(|c| c.text.clone()))
                } else {
                    m.single.get(name).map(|c| c.text.clone())
                };
                if let Some(v) = value {
                    out.push_str(&v);
                    i = name_start + name_len;
                    continue;
                }
                if dollars == 3 && m.multi.contains_key(name) {
                    i = name_start + name_len;
                    continue;
                }
            }
            out.push_str(&template[i..name_start]);
            i = name_start;
            continue;
        }
        let ch = template[i..].chars().next().expect("in bounds");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

#[cfg(test)]
mod tests;
