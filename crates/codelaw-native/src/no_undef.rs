//! `js/no-undef`: a reference nothing declares — not the file, not the
//! language, not the environments the rule's `env` option names. The
//! environments' globals are the pack's `globals/<env>` data tables.

use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use std::collections::BTreeSet;

pub struct NoUndef;

impl NativeRule for NoUndef {
    fn id(&self) -> &'static str {
        "js/no-undef"
    }
    fn languages(&self) -> &'static [Language] {
        &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["program"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let root = node.get_inner_node();
        let check_typeof = ctx.options.get("typeof").and_then(|v| v.as_bool()).unwrap_or(false);
        let mut known: BTreeSet<String> = BTreeSet::new();
        for env in ctx.options.get("env").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|v| v.as_str()) {
            match ctx.data.get(&format!("globals/{env}")).map(|v| v.as_ref()) {
                Some(serde_json::Value::Object(o)) => known.extend(o.keys().cloned()),
                Some(serde_json::Value::Array(a)) => known.extend(a.iter().filter_map(|x| x.as_str().map(str::to_owned))),
                _ => {}
            }
        }
        for g in ctx.options.get("globals").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|v| v.as_str()) {
            known.insert(g.to_owned());
        }
        for r in &model.references {
            if r.type_only || !model.is_undeclared(r, &|n| known.contains(n)) {
                continue;
            }
            if !check_typeof
                && let Some(n) = root.descendant_for_byte_range(r.start, r.end)
                && let Some(p) = n.parent()
                && p.kind() == "unary_expression"
                && p.child(0).is_some_and(|op| op.kind() == "typeof")
            {
                continue;
            }
            out.push(NativeFinding { start: r.start, end: r.end, message: format!("`{}` is not defined", r.name), edits: Vec::new() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    #[test]
    fn environments_come_from_data_tables() {
        let src = "window.x; process.exit(); Object.keys(o); typeof maybe; let o = {};";
        let root = codelaw_lang::parse(Language::Javascript, src);
        let model = codelaw_semantic::build(Language::Javascript, &root, src);
        let mut data = BTreeMap::new();
        data.insert("globals/browser".to_owned(), Arc::new(serde_json::json!({ "window": false })));
        let mut options = BTreeMap::new();
        options.insert("env".to_owned(), serde_json::json!(["browser"]));
        let ctx = NativeContext { source: src, language: Language::Javascript, semantic: model.as_ref(), options: &options, data: &data };
        let mut out = Vec::new();
        NoUndef.check(root.root(), &ctx, &mut out);
        let names: Vec<&str> = out.iter().map(|f| &src[f.start..f.end]).collect();
        assert_eq!(names, ["process"]);
    }
}
