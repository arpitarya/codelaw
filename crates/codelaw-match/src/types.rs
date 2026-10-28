//! `type:` — the type questions (SR-RULE-FORMAT decision 3f; SR-TYPES
//! decision 4). A clause maps a capture to a question from the fixed
//! vocabulary, or `not`, `all` and `any` over them:
//!
//! ```yaml
//! type:
//!   $CALL: promise-like
//!   $X: { not: { any: [thenable, any] } }
//! ```
//!
//! The answers come from the project's own TypeScript, through the engine's
//! helper; this module only says which questions a match asks and whether
//! it holds once they are answered.

use crate::{MATCH_KEY, Match};
use serde_json::Value;

/// A question, or `not`, `all` and `any` over questions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeExpr {
    Is(&'static str),
    Not(Box<TypeExpr>),
    All(Vec<TypeExpr>),
    Any(Vec<TypeExpr>),
}

impl TypeExpr {
    fn parse(v: &Value) -> Result<TypeExpr, String> {
        match v {
            Value::String(s) => match codelaw_types::QUESTIONS.iter().find(|q| **q == s.as_str()) {
                Some(q) => Ok(TypeExpr::Is(q)),
                None => {
                    let near = codelaw_config::did_you_mean(s, codelaw_types::QUESTIONS.iter().copied()).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
                    Err(format!("`{s}` is not a type question (SR-TYPES decision 4: {}){near}", codelaw_types::QUESTIONS.join(", ")))
                }
            },
            // A list means all of them, as in `where:`.
            Value::Array(items) => Ok(TypeExpr::All(items.iter().map(TypeExpr::parse).collect::<Result<_, _>>()?)),
            Value::Object(o) if o.len() == 1 => {
                let (k, x) = o.iter().next().expect("one key");
                match k.as_str() {
                    "not" => Ok(TypeExpr::Not(Box::new(TypeExpr::parse(x)?))),
                    "all" | "any" => {
                        let Value::Array(items) = x else { return Err(format!("`{k}:` takes a list of type questions")) };
                        let parts = items.iter().map(TypeExpr::parse).collect::<Result<Vec<_>, _>>()?;
                        Ok(if k == "all" { TypeExpr::All(parts) } else { TypeExpr::Any(parts) })
                    }
                    other => Err(format!("`{other}:` is not a type question; a clause is a question, or `not`, `all` or `any` over them")),
                }
            }
            _ => Err("a `type:` clause is a question, or `not`, `all` or `any` over them".into()),
        }
    }

    fn questions(&self, out: &mut Vec<&'static str>) {
        match self {
            TypeExpr::Is(q) => out.push(q),
            TypeExpr::Not(x) => x.questions(out),
            TypeExpr::All(xs) | TypeExpr::Any(xs) => xs.iter().for_each(|x| x.questions(out)),
        }
    }

    fn holds(&self, answer: &dyn Fn(&'static str) -> bool) -> bool {
        match self {
            TypeExpr::Is(q) => answer(q),
            TypeExpr::Not(x) => !x.holds(answer),
            TypeExpr::All(xs) => xs.iter().all(|x| x.holds(answer)),
            TypeExpr::Any(xs) => xs.iter().any(|x| x.holds(answer)),
        }
    }
}

/// Parse a `type:` block: captures (`$NAME`, `$$MATCH`) to clauses.
pub fn parse_types(v: &Value) -> Result<Vec<(String, TypeExpr)>, String> {
    let Value::Object(map) = v else { return Err("`type:` maps captures (`$NAME`, `$$MATCH`) to type questions".into()) };
    map.iter().map(|(k, x)| Ok((k.clone(), TypeExpr::parse(x)?))).collect()
}

/// The span a clause's key names in a match, or `None` when the matched
/// branch did not bind it.
fn span_of(m: &Match, var: &str) -> Option<(usize, usize)> {
    if var == MATCH_KEY {
        return Some((m.start, m.end));
    }
    m.single.get(var.trim_start_matches('$')).map(|c| (c.start, c.end))
}

/// Every question a match asks: `(start, end, question)`, UTF-8 offsets.
pub fn questions_of(clauses: &[(String, TypeExpr)], m: &Match) -> Vec<(usize, usize, &'static str)> {
    let mut out = Vec::new();
    for (var, expr) in clauses {
        let Some((s, e)) = span_of(m, var) else { continue };
        let mut qs = Vec::new();
        expr.questions(&mut qs);
        out.extend(qs.into_iter().map(|q| (s, e, q)));
    }
    out
}

/// Whether every clause holds, given the answers. A question with no
/// answer (no node spans it) does not hold, even under `not`: a type rule
/// never reports on a guess (decision 3f). A clause on a capture the
/// matched branch did not bind is skipped, as `calls:` is: that branch has
/// no node to ask about.
pub fn types_hold(clauses: &[(String, TypeExpr)], m: &Match, answer: &dyn Fn(usize, usize, &'static str) -> Option<bool>) -> bool {
    clauses.iter().all(|(var, expr)| match span_of(m, var) {
        Some((s, e)) => {
            let unknown = std::cell::Cell::new(false);
            let holds = expr.holds(&|q| {
                answer(s, e, q).unwrap_or_else(|| {
                    unknown.set(true);
                    false
                })
            });
            holds && !unknown.get()
        }
        None => true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn the_vocabulary_is_fixed() {
        let ok = parse_types(&json!({ "$X": { "not": { "any": ["thenable", "any"] } }, "$$MATCH": ["union", "nullable"] })).unwrap();
        assert_eq!(ok.len(), 2);
        let e = parse_types(&json!({ "$X": "promise" })).unwrap_err();
        assert!(e.contains("not a type question") && e.contains("did you mean `promise-like`"), "{e}");
        assert!(parse_types(&json!({ "$X": { "is": "any" } })).is_err());
        assert!(parse_types(&json!(["any"])).is_err());
    }

    #[test]
    fn a_match_holds_only_when_every_answer_is_known_and_true() {
        let clauses = parse_types(&json!({ "$X": { "not": "thenable" } })).unwrap();
        let x = crate::Captured { start: 6, end: 7, text: "x".into(), kind: "identifier".into() };
        let m = Match { start: 0, end: 7, kind: "await_expression".into(), single: BTreeMap::from([("X".to_owned(), x)]), multi: BTreeMap::new() };
        assert_eq!(questions_of(&clauses, &m), vec![(6, 7, "thenable")]);
        assert!(types_hold(&clauses, &m, &|_, _, _| Some(false)));
        assert!(!types_hold(&clauses, &m, &|_, _, _| Some(true)));
        // No node spans it: never a guess, even under `not`.
        assert!(!types_hold(&clauses, &m, &|_, _, _| None));
        // A branch that did not bind `$Y` has nothing to ask about it.
        let unbound = parse_types(&json!({ "$X": "any", "$Y": { "not": "any" } })).unwrap();
        assert!(types_hold(&unbound, &m, &|_, _, _| Some(true)));
    }
}
