//! `codelaw match <id> <file> --explain` (SR-RULE-AUTHORING decision 3):
//! which clause of a `match:` rule held or failed on each candidate node,
//! what it captured, and which `where:` predicate decided.

use crate::{CompiledMatch, MATCH_KEY, captured, compile, rule_core};
use ast_grep_config::RuleCore;
use ast_grep_core::matcher::MatcherExt;
use codelaw_lang::{Grammar, Root, SyntaxNode};
use codelaw_rules::{BlockKind, RuleFile};
use codelaw_semantic::SemanticModel;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Keys that pick nodes by themselves; a node is a candidate when one of
/// them holds. Relational clauses and `not` only narrow.
const ANCHORS: &[&str] = &["pattern", "kind", "regex", "nthChild", "range", "matches", "any"];

struct Clause {
    label: String,
    anchor: bool,
    core: Result<RuleCore, String>,
}

/// One clause on one node: held, failed, or could not be evaluated alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClauseResult {
    pub label: String,
    /// `None`: the clause cannot be evaluated on its own (the reason is in
    /// `error`).
    pub holds: Option<bool>,
    pub error: Option<String>,
}

/// One candidate node and why the rule did or did not match it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeExplanation {
    pub start: usize,
    pub end: usize,
    pub kind: String,
    pub clauses: Vec<ClauseResult>,
    /// `Some(false)`: every clause held but `constraints:` failed.
    pub constraints: Option<bool>,
    /// What the rule (or the clauses that held) captured: `$NAME` → text.
    pub captures: BTreeMap<String, String>,
    /// Each `where:` predicate, evaluated when the pattern part matched.
    pub predicates: Vec<ClauseResult>,
    pub matched: bool,
}

/// A rule's `match:` block split into its clauses, for one grammar.
pub struct Explainer {
    full: CompiledMatch,
    without_constraints: Option<RuleCore>,
    clauses: Vec<Clause>,
    predicate_labels: Vec<String>,
}

fn label(key: &str, v: &Value) -> String {
    let mut s = format!("{key}: {}", serde_json::to_string(v).unwrap_or_default());
    if s.chars().count() > 72 {
        s = s.chars().take(71).collect::<String>() + "…";
    }
    s
}

impl Explainer {
    pub fn new(rule: &RuleFile, grammar: Grammar, data: &BTreeMap<String, Arc<Value>>) -> Result<Explainer, String> {
        let full = compile(rule, grammar, data)?;
        let BlockKind::Match { rule: matcher, constraints, utils, predicates } = &rule.block.kind else {
            return Err("not a `match:` rule".into());
        };
        let mut parts: Vec<(String, Value)> = Vec::new();
        if let Value::Object(o) = matcher {
            for (k, v) in o {
                match (k.as_str(), v) {
                    ("all", Value::Array(items)) => {
                        for (i, item) in items.iter().enumerate() {
                            let key = item.as_object().and_then(|o| o.keys().next().cloned()).unwrap_or_default();
                            parts.push((format!("all[{i}].{key}"), item.clone()));
                        }
                    }
                    _ => parts.push((k.clone(), serde_json::json!({ k: v }))),
                }
            }
        }
        let anchor_of = |name: &str| ANCHORS.iter().any(|a| name == *a || name.ends_with(&format!(".{a}")));
        let any_anchor = parts.iter().any(|(n, _)| anchor_of(n));
        let clauses = parts
            .into_iter()
            .map(|(name, v)| {
                let inner = v.as_object().and_then(|o| o.values().next()).cloned().unwrap_or(Value::Null);
                let shown = if name.starts_with("all[") { label(&name, inner.as_object().and_then(|o| o.values().next()).unwrap_or(&inner)) } else { label(&name, &inner) };
                let anchor = if any_anchor { anchor_of(&name) } else { !name.ends_with("not") };
                Clause { label: shown, anchor, core: rule_core(&v, None, utils.as_ref(), grammar) }
            })
            .collect();
        let without_constraints = match constraints {
            Some(_) => Some(rule_core(matcher, None, utils.as_ref(), grammar)?),
            None => None,
        };
        let predicate_labels = match predicates {
            Some(Value::Object(m)) => m.iter().map(|(var, spec)| label(var, spec)).collect(),
            _ => Vec::new(),
        };
        Ok(Explainer { full, without_constraints, clauses, predicate_labels })
    }

    fn captures(core: &RuleCore, node: &SyntaxNode<'_>, out: &mut BTreeMap<String, String>) -> bool {
        let Some(m) = core.match_node(node.clone()) else { return false };
        let env = m.get_env();
        for var in core.defined_vars() {
            if let Some(n) = env.get_match(var) {
                out.entry(format!("${var}")).or_insert_with(|| n.text().to_string());
            } else {
                let nodes = env.get_multiple_matches(var);
                if !nodes.is_empty() {
                    let text = nodes.iter().map(|n| n.text().to_string()).collect::<Vec<_>>().join(", ");
                    out.entry(format!("$$${var}")).or_insert(text);
                }
            }
        }
        true
    }

    /// Explain one node; `None` when it is not a candidate (no anchor
    /// clause holds and the rule does not match).
    pub fn explain_node(&self, node: SyntaxNode<'_>, semantic: Option<&SemanticModel>) -> Option<NodeExplanation> {
        let mut captures = BTreeMap::new();
        let mut clauses = Vec::new();
        let mut anchored = false;
        let mut all_hold = true;
        for c in &self.clauses {
            let (holds, error) = match &c.core {
                Ok(core) => (Some(Self::captures(core, &node, &mut captures)), None),
                Err(e) => (None, Some(e.clone())),
            };
            if holds == Some(true) && c.anchor {
                anchored = true;
            }
            if holds == Some(false) {
                all_hold = false;
            }
            clauses.push(ClauseResult { label: c.label.clone(), holds, error });
        }
        let core_match = self.full.core.match_node(node.clone()).is_some();
        if !anchored && !core_match {
            return None;
        }
        let constraints = self.without_constraints.as_ref().map(|wc| !(all_hold && wc.match_node(node.clone()).is_some() && !core_match));
        let mut predicates = Vec::new();
        let mut matched = core_match;
        if core_match {
            let mut full_caps = BTreeMap::new();
            Self::captures(&self.full.core, &node, &mut full_caps);
            captures = full_caps;
            let this = captured(&node);
            let m = self.full.core.match_node(node.clone()).expect("matched");
            let env = m.get_env();
            let source: &str = {
                use ast_grep_core::Doc;
                m.get_doc().get_source()
            };
            for ((var, p), label) in self.full.predicates.iter().zip(&self.predicate_labels) {
                let target = if var == MATCH_KEY { Some(m.get_inner_node()) } else { env.get_match(var.trim_start_matches('$')).map(|n| n.get_inner_node()) };
                let start = target.map(|t| t.start_byte()).or((var == MATCH_KEY).then_some(this.start));
                let holds = match (target, semantic) {
                    (Some(t), Some(model)) => Some(p.holds(model, &crate::Subject { start: t.start_byte(), node: Some(t), source, options: &BTreeMap::new() })),
                    _ => None,
                };
                let error = match (start, semantic) {
                    (None, _) => Some(format!("`{var}` captured nothing")),
                    (_, None) => Some("no semantic model for this file".to_owned()),
                    _ => None,
                };
                if holds != Some(true) {
                    matched = false;
                }
                predicates.push(ClauseResult { label: label.clone(), holds, error });
            }
        }
        let r = node.range();
        Some(NodeExplanation { start: r.start, end: r.end, kind: node.kind().to_string(), clauses, constraints, captures, predicates, matched })
    }

    /// Every candidate node of a parsed file, in document order.
    pub fn explain(&self, root: &Root, semantic: Option<&SemanticModel>) -> Vec<NodeExplanation> {
        root.root().dfs().filter_map(|n| self.explain_node(n, semantic)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codelaw_core::Language;
    use std::path::Path;

    fn rule(block: &str) -> RuleFile {
        let text = format!(
            "---\nid: t/r\ntitle: t\nlanguages: [javascript]\nfix: manual\nfix_reason: test\n---\n\nSummary.\n\n**Why:** test.\n\n```codelaw\n{block}\n```\n\n## How to fix\n\n1. Fix it.\n\n## Examples\n\n### JavaScript\n\n#### Don't\n\n```js\nx\n```\n\n#### Do\n\n```js\ny\n```\n"
        );
        codelaw_rules::parse_rule(Path::new("t/r.md"), &text, Some("t/r")).unwrap_or_else(|e| panic!("{e:?}"))
    }

    #[test]
    fn names_the_clause_that_failed() {
        let r = rule("match:\n  pattern: console.log($A)\n  inside: { kind: function_declaration, stopBy: end }\nmessage: m");
        let ex = Explainer::new(&r, Grammar::JavaScript, &BTreeMap::new()).unwrap();
        let src = "console.log(1);\nfunction f() { console.log(2); }\n";
        let root = codelaw_lang::parse(Language::Javascript, src);
        let out = ex.explain(&root, None);
        assert_eq!(out.len(), 2, "{out:#?}");
        assert!(!out[0].matched);
        assert_eq!(out[0].clauses[0].holds, Some(true));
        assert!(out[0].clauses[1].label.starts_with("inside: "), "{:?}", out[0].clauses);
        assert_eq!(out[0].clauses[1].holds, Some(false));
        assert_eq!(out[0].captures["$A"], "1");
        assert!(out[1].matched);
        assert_eq!(out[1].captures["$A"], "2");
    }

    #[test]
    fn names_the_predicate_that_failed() {
        let r = rule("match:\n  kind: identifier\n  inside: { kind: variable_declarator, field: name }\nwhere:\n  $$MATCH: unused\nmessage: m");
        let ex = Explainer::new(&r, Grammar::JavaScript, &BTreeMap::new()).unwrap();
        let src = "const a = 1;\nconst b = 2;\nf(b);\n";
        let root = codelaw_lang::parse(Language::Javascript, src);
        let model = codelaw_semantic::build(Language::Javascript, &root, src);
        let out: Vec<_> = ex.explain(&root, model.as_ref()).into_iter().filter(|n| n.clauses.iter().all(|c| c.holds == Some(true))).collect();
        assert_eq!(out.len(), 2, "{out:#?}");
        assert!(out[0].matched);
        assert_eq!(out[1].predicates[0].holds, Some(false));
        assert!(!out[1].matched);
    }
}
