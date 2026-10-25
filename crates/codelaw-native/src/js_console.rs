//! `js/no-console`: calls on the global `console` (`console.log(…)`,
//! `console["log"](…)`), not on a local that shadows it. The `allow` option
//! names methods that may stay (`["warn", "error"]`). The fix deletes the
//! statement when the call is the whole of one in a block; anywhere else
//! the call's value may be used, so the fix is the author's.

use crate::util::{named_children, string_value, text, whole_lines};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use tree_sitter::Node;

pub struct NoConsole;

/// The method a callee names on `console`: `console.log` → `log`,
/// `console["log"]` → `log`; `None` when it is not a member of an
/// identifier `console`, or the member is computed.
fn console_method<'t>(callee: Node<'t>, src: &str) -> Option<(Node<'t>, String)> {
    let object = callee.child_by_field_name("object")?;
    if object.kind() != "identifier" || text(object, src) != "console" {
        return None;
    }
    let method = match callee.kind() {
        "member_expression" => text(callee.child_by_field_name("property")?, src).to_owned(),
        "subscript_expression" => {
            let index = callee.child_by_field_name("index")?;
            if index.kind() != "string" {
                return None;
            }
            string_value(index, src)
        }
        _ => return None,
    };
    Some((object, method))
}

/// The statement to delete: the call is the whole of an expression
/// statement that sits in a list of statements.
fn removable(call: Node<'_>) -> Option<Node<'_>> {
    let stmt = call.parent().filter(|p| p.kind() == "expression_statement")?;
    if named_children(stmt).len() != 1 {
        return None;
    }
    let list = stmt.parent()?;
    matches!(list.kind(), "program" | "statement_block" | "switch_case" | "switch_default" | "class_static_block").then_some(stmt)
}

impl NativeRule for NoConsole {
    fn id(&self) -> &'static str {
        "js/no-console"
    }
    fn languages(&self) -> &'static [Language] {
        &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let n = node.get_inner_node();
        let src = ctx.source;
        let Some(callee) = n.child_by_field_name("function") else { return };
        let Some((object, method)) = console_method(callee, src) else { return };
        // A local `console` is not the global one.
        if model.reference_at(object.start_byte()).is_none_or(|r| r.symbol.is_some()) {
            return;
        }
        let allowed = ctx.options.get("allow").and_then(|v| v.as_array()).is_some_and(|a| a.iter().any(|m| m.as_str() == Some(method.as_str())));
        if allowed {
            return;
        }
        let edits = removable(n)
            .map(|stmt| {
                let (s, e) = whole_lines(src, stmt.start_byte(), stmt.end_byte());
                vec![TextEdit::new(s, e, "")]
            })
            .unwrap_or_default();
        out.push(NativeFinding {
            start: n.start_byte(),
            end: n.end_byte(),
            message: format!("`console.{method}` left in the code; remove it or report through a logger"),
            edits,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn run(src: &str, options: serde_json::Value) -> Vec<NativeFinding> {
        let root = codelaw_lang::parse(Language::Javascript, src);
        let model = codelaw_semantic::build(Language::Javascript, &root, src);
        let options: BTreeMap<String, serde_json::Value> = serde_json::from_value(options).unwrap();
        let data = BTreeMap::new();
        let ctx = NativeContext { source: src, language: Language::Javascript, semantic: model.as_ref(), options: &options, data: &data, file: &crate::DetachedFile::default() };
        let mut out = Vec::new();
        for n in root.root().dfs().filter(|n| n.kind() == "call_expression") {
            NoConsole.check(n, &ctx, &mut out);
        }
        out
    }

    #[test]
    fn global_console_calls() {
        let src = "console.log(1);\nconsole[\"debug\"](2);\nconsole.warn(3);\n";
        let out = run(src, serde_json::json!({}));
        assert_eq!(out.len(), 3, "{out:?}");
        assert_eq!(out[0].edits, vec![TextEdit::new(0, 16, "")]);
        let out = run(src, serde_json::json!({ "allow": ["warn", "debug"] }));
        assert_eq!(out.iter().map(|f| f.message.as_str()).collect::<Vec<_>>(), ["`console.log` left in the code; remove it or report through a logger"]);
    }
}
