//! The `call[]` option type (SR-RULE-FORMAT decision 2, W-73): a list of
//! approved functions, each a callee path or `{ name, module }`, with a
//! `<option>_mode` of `extend` (the project's list is added to the rule's
//! default) or `replace` (it is used instead).

use serde_json::Value;
use std::collections::BTreeMap;

/// One approved function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallItem {
    /// A callee path: `DOMPurify.sanitize`, `*.createHTML` for any
    /// receiver, or `*Schema.parse` for a receiver ending in `Schema`.
    pub name: String,
    /// The modules the root binding must be imported from; empty means the
    /// root is a global (undeclared in the file) - or, for a `*` or
    /// `*<suffix>` root, any.
    pub modules: Vec<String>,
}

/// The suffix of the option that carries a `call[]` option's mode.
pub const MODE_SUFFIX: &str = "_mode";

/// The internal type of a `<opt>_mode` option.
pub const MODE_KIND: &str = "extend|replace";

/// Why `v` is not a `call[]`, or its items.
pub fn parse_call_items(v: &Value) -> Result<Vec<CallItem>, String> {
    let Value::Array(items) = v else { return Err("a `call[]` is a list".into()) };
    items.iter().map(parse_item).collect()
}

fn valid_path(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    // A segment is a name, `*` (any one segment) or `*<suffix>` (one ending
    // in the suffix).
    let name = |p: &str| !p.is_empty() && p.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$');
    !parts.is_empty() && parts.iter().all(|p| *p == "*" || name(p.strip_prefix('*').unwrap_or(p)))
}

fn parse_item(v: &Value) -> Result<CallItem, String> {
    match v {
        Value::String(s) if valid_path(s) => Ok(CallItem { name: s.clone(), modules: Vec::new() }),
        Value::String(s) => Err(format!("`{s}` is not a callee path like `DOMPurify.sanitize`")),
        Value::Object(o) => {
            if let Some(k) = o.keys().find(|k| *k != "name" && *k != "module") {
                return Err(format!("a call takes `name` and `module`, not `{k}`"));
            }
            let name = match o.get("name") {
                Some(Value::String(s)) if valid_path(s) && !s.starts_with('*') => s.clone(),
                Some(Value::String(s)) if valid_path(s) => return Err(format!("`{s}` matches any root, so it takes no `module`; write it as a bare path")),
                Some(_) => return Err("a call's `name` is a callee path like `DOMPurify.sanitize`".into()),
                None => return Err("a call needs a `name`".into()),
            };
            let modules = match o.get("module") {
                Some(Value::String(m)) if !m.is_empty() => vec![m.clone()],
                Some(Value::Array(a)) if !a.is_empty() && a.iter().all(|m| m.as_str().is_some_and(|m| !m.is_empty())) => {
                    a.iter().filter_map(|m| m.as_str().map(str::to_owned)).collect()
                }
                Some(_) => return Err("a call's `module` is a module or a non-empty list of modules".into()),
                None => return Err("a call needs a `module` (write a bare callee path for a global)".into()),
            };
            Ok(CallItem { name, modules })
        }
        _ => Err("a call is a callee path or `{ name, module }`".into()),
    }
}

/// Whether the option's mode says `replace`.
pub fn is_replace(options: &BTreeMap<String, Value>, option: &str) -> bool {
    options.get(&format!("{option}{MODE_SUFFIX}")).and_then(Value::as_str) == Some("replace")
}

/// The list a rule runs with: the rule's default, plus the project's items
/// (`extend`) or the project's items alone (`replace`). `options` holds the
/// default where the project set nothing.
pub fn resolve_calls(default: &[CallItem], options: &BTreeMap<String, Value>, option: &str) -> Vec<CallItem> {
    let project = options.get(option).and_then(|v| parse_call_items(v).ok());
    let mut out: Vec<CallItem> = match project {
        None => default.to_vec(),
        Some(p) if is_replace(options, option) => p,
        Some(p) => default.iter().cloned().chain(p).collect(),
    };
    let mut seen = Vec::new();
    out.retain(|c| {
        let fresh = !seen.contains(c);
        if fresh {
            seen.push(c.clone());
        }
        fresh
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn items_are_paths_or_name_and_module() {
        let items = parse_call_items(&json!(["DOMPurify.sanitize", "*.createHTML", { "name": "clean", "module": ["a", "b"] }, "*Schema.parse"])).unwrap();
        assert_eq!(items[2], CallItem { name: "clean".into(), modules: vec!["a".into(), "b".into()] });
        assert!(items[0].modules.is_empty());
    }

    #[test]
    fn a_bad_item_is_refused() {
        for bad in [json!("not a path()"), json!({ "name": "x" }), json!({ "name": "x", "module": 3 }), json!({ "name": "x", "module": "m", "extra": 1 }), json!(7), json!(""), json!("a.**"), json!("*-x.parse"), json!({ "name": "*.parse", "module": "m" })] {
            assert!(parse_call_items(&json!([bad])).is_err());
        }
        assert!(parse_call_items(&json!("x")).is_err());
    }

    #[test]
    fn extend_adds_to_the_default_and_replace_uses_the_project_list() {
        let default = parse_call_items(&json!([{ "name": "d", "module": "m" }])).unwrap();
        let mut o = BTreeMap::new();
        o.insert("s".to_owned(), json!([{ "name": "mine", "module": "p" }]));
        assert_eq!(resolve_calls(&default, &o, "s").len(), 2);
        o.insert("s_mode".to_owned(), json!("replace"));
        let r = resolve_calls(&default, &o, "s");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].name, "mine");
        assert_eq!(resolve_calls(&default, &BTreeMap::new(), "s"), default);
    }
}
