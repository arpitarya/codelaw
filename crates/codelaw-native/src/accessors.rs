//! Getters must return a value on every path (`js/getter-return`); setters
//! must not return one (`js/no-setter-return`). Both need the paths through
//! the body, which a pattern can't follow.

use crate::util::{descendants, has_token, is_function, returns_value};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use tree_sitter::Node;

const JS: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];

fn accessor_body<'t>(n: Node<'t>, token: &str) -> Option<(Node<'t>, Node<'t>)> {
    if n.kind() != "method_definition" || !has_token(n, token) {
        return None;
    }
    Some((n.child_by_field_name("name")?, n.child_by_field_name("body")?))
}

pub struct GetterReturn;

impl NativeRule for GetterReturn {
    fn id(&self) -> &'static str {
        "js/getter-return"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["method_definition"]
    }
    fn check(&self, node: SyntaxNode<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some((name, body)) = accessor_body(node.get_inner_node(), "get") else { return };
        if !returns_value(&body) {
            out.push(NativeFinding { start: name.start_byte(), end: name.end_byte(), message: "This getter doesn't return a value on every path".into(), edits: Vec::new() });
        }
    }
}

pub struct NoSetterReturn;

impl NativeRule for NoSetterReturn {
    fn id(&self) -> &'static str {
        "js/no-setter-return"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["method_definition"]
    }
    fn check(&self, node: SyntaxNode<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some((_, body)) = accessor_body(node.get_inner_node(), "set") else { return };
        let mut inside = Vec::new();
        descendants(body, &|c| is_function(c.kind()) || c.kind() == "arrow_function", &mut inside);
        for r in inside.into_iter().filter(|r| r.kind() == "return_statement" && r.named_child_count() > 0) {
            out.push(NativeFinding { start: r.start_byte(), end: r.end_byte(), message: "A setter's return value is ignored; don't return one".into(), edits: Vec::new() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(rule: &dyn NativeRule, src: &str) -> usize {
        crate::run(rule, Language::Javascript, src).len()
    }

    #[test]
    fn getters_and_setters() {
        assert_eq!(count(&GetterReturn, "o = { get a() {}, get b() { if (x) return 1; }, get c() { return 1; }, get d() { if (x) return 1; else throw e; } };"), 2);
        assert_eq!(count(&GetterReturn, "class A { get a() { switch (x) { case 1: return 1; default: return 2; } } get b() { return; } }"), 1);
        assert_eq!(count(&NoSetterReturn, "class A { set a(v) { return v; } set b(v) { if (v) return; this.x = () => v; } }"), 1);
    }
}
