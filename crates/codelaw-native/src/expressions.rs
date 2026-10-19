//! Rules that reason about what an expression can evaluate to:
//! `js/no-constant-binary-expression` (an operand whose truthiness or
//! nullishness is fixed, or a comparison with a brand-new object) and
//! `js/no-unsafe-optional-chaining` (a `?.` chain used where `undefined`
//! throws).

use crate::util::{children, named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use tree_sitter::Node;

const JS: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];

fn unwrap_parens(mut n: Node<'_>) -> Node<'_> {
    while n.kind() == "parenthesized_expression" {
        match n.named_child(0) {
            Some(inner) => n = inner,
            None => break,
        }
    }
    n
}

fn operator<'s>(n: Node<'_>, src: &'s str) -> &'s str {
    n.child_by_field_name("operator").map(|o| text(o, src)).unwrap_or("")
}

/// An expression that creates a new object every time it runs.
fn new_object(n: Node<'_>) -> bool {
    matches!(
        unwrap_parens(n).kind(),
        "object" | "array" | "arrow_function" | "function_expression" | "function" | "class" | "new_expression" | "regex"
    )
}

/// `Some(true)` if never nullish, `Some(false)` if always nullish.
fn nullishness(n: Node<'_>, src: &str) -> Option<bool> {
    let n = unwrap_parens(n);
    match n.kind() {
        "null" | "undefined" => Some(false),
        "unary_expression" => Some(operator(n, src) != "void"),
        "binary_expression" => match operator(n, src) {
            "??" | "||" | "&&" => None,
            _ => Some(true),
        },
        "number" | "string" | "template_string" | "true" | "false" | "update_expression" => Some(true),
        _ if new_object(n) => Some(true),
        _ => None,
    }
}

/// `Some(true)` if always truthy, `Some(false)` if always falsy.
fn truthiness(n: Node<'_>, src: &str) -> Option<bool> {
    let n = unwrap_parens(n);
    match n.kind() {
        "true" => Some(true),
        "false" | "null" | "undefined" => Some(false),
        "number" => Some(!matches!(text(n, src).trim_start_matches('-'), "0" | "0.0" | "0x0" | "0n")),
        "string" => Some(text(n, src).len() > 2),
        "unary_expression" if operator(n, src) == "void" => Some(false),
        _ if new_object(n) => Some(true),
        _ => None,
    }
}

pub struct NoConstantBinaryExpression;

impl NativeRule for NoConstantBinaryExpression {
    fn id(&self) -> &'static str {
        "js/no-constant-binary-expression"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["binary_expression"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let src = ctx.source;
        let (Some(left), Some(right)) = (n.child_by_field_name("left"), n.child_by_field_name("right")) else { return };
        let op = operator(n, src);
        let message = match op {
            "??" => nullishness(left, src).map(|never| {
                if never { "The left side of `??` is never null or undefined, so the right side never runs" } else { "The left side of `??` is always null or undefined" }
            }),
            "||" | "&&" => truthiness(left, src).map(|_| "The left side of this logical expression is a constant, so its result is fixed"),
            "==" | "===" | "!=" | "!==" => {
                if new_object(left) || new_object(right) {
                    Some("A newly created object is never equal to anything else, so this comparison is constant")
                } else {
                    let is_nullish_literal = |x: Node<'_>| matches!(unwrap_parens(x).kind(), "null" | "undefined");
                    let other = if is_nullish_literal(right) { Some(left) } else if is_nullish_literal(left) { Some(right) } else { None };
                    other.and_then(|o| nullishness(o, src)).filter(|never| *never).map(|_| "One side can never be null or undefined, so this comparison is constant")
                }
            }
            _ => None,
        };
        if let Some(m) = message {
            out.push(NativeFinding { start: n.start_byte(), end: n.end_byte(), message: m.into(), edits: Vec::new() });
        }
    }
}

/// Whether evaluating `n` can short-circuit to `undefined` through `?.`.
fn optional_chain(n: Node<'_>) -> bool {
    let n = unwrap_parens(n);
    match n.kind() {
        "member_expression" | "subscript_expression" => {
            children(n).iter().any(|c| c.kind() == "optional_chain") || n.child_by_field_name("object").is_some_and(optional_chain)
        }
        "call_expression" => children(n).iter().any(|c| c.kind() == "optional_chain") || n.child_by_field_name("function").is_some_and(optional_chain),
        "await_expression" => n.named_child(0).is_some_and(optional_chain),
        _ => false,
    }
}

pub struct NoUnsafeOptionalChaining;

impl NativeRule for NoUnsafeOptionalChaining {
    fn id(&self) -> &'static str {
        "js/no-unsafe-optional-chaining"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &[
            "call_expression",
            "new_expression",
            "member_expression",
            "subscript_expression",
            "spread_element",
            "variable_declarator",
            "assignment_expression",
            "for_in_statement",
            "binary_expression",
            "class_heritage",
        ]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let src = ctx.source;
        // The sub-expression that must not be undefined, if this node uses one.
        let target = match n.kind() {
            // `(a?.b)()` and `(a?.b).c`: a parenthesised chain doesn't
            // short-circuit the call or access around it.
            "call_expression" => n.child_by_field_name("function").filter(|f| f.kind() == "parenthesized_expression"),
            "member_expression" | "subscript_expression" => n.child_by_field_name("object").filter(|f| f.kind() == "parenthesized_expression"),
            "new_expression" => n.child_by_field_name("constructor"),
            "spread_element" => n.parent().filter(|p| matches!(p.kind(), "array" | "arguments")).and_then(|_| n.named_child(0)),
            "variable_declarator" => n
                .child_by_field_name("name")
                .filter(|p| matches!(p.kind(), "object_pattern" | "array_pattern"))
                .and_then(|_| n.child_by_field_name("value")),
            "assignment_expression" => n
                .child_by_field_name("left")
                .filter(|p| matches!(p.kind(), "object_pattern" | "array_pattern"))
                .and_then(|_| n.child_by_field_name("right")),
            "for_in_statement" => children(n).iter().any(|c| c.kind() == "of").then(|| n.child_by_field_name("right")).flatten(),
            "binary_expression" => matches!(operator(n, src), "in" | "instanceof").then(|| n.child_by_field_name("right")).flatten(),
            "class_heritage" => named_children(n).into_iter().next().map(|c| if c.kind() == "extends_clause" { c.named_child(0).unwrap_or(c) } else { c }),
            _ => None,
        };
        if let Some(t) = target
            && optional_chain(t)
        {
            out.push(NativeFinding {
                start: t.start_byte(),
                end: t.end_byte(),
                message: format!("`{}` can be undefined here, and using it this way throws a TypeError", text(t, src)),
                edits: Vec::new(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(rule: &dyn NativeRule, src: &str) -> Vec<String> {
        crate::run(rule, Language::Javascript, src).into_iter().map(|f| src[f.start..f.end].to_owned()).collect()
    }

    #[test]
    fn constant_binary() {
        let src = "a = x + 1 ?? 0; b = {} || c; d = y === []; e = (z + 1) == null; f = x ?? y; g = p === null; h = void 0 ?? q; i = s || t;";
        assert_eq!(found(&NoConstantBinaryExpression, src), ["x + 1 ?? 0", "{} || c", "y === []", "(z + 1) == null", "void 0 ?? q"]);
    }

    #[test]
    fn unsafe_optional_chaining() {
        let src = "(a?.b)(); (a?.b).c; new a?.b(); [...a?.b]; f(...a?.b); const { x } = a?.b; for (const v of a?.list) {} k in a?.b; class C extends a?.B {}\nok = a?.b(); ok2 = a?.b.c; ok3 = { ...a?.b };";
        assert_eq!(found(&NoUnsafeOptionalChaining, src), ["(a?.b)", "(a?.b)", "a?.b", "a?.b", "a?.b", "a?.b", "a?.list", "a?.b", "a?.B"]);
    }
}
