//! `js/no-unreachable`: statements after a `return`, `throw`, `break` or
//! `continue` in the same block never run. A pattern can match the exit,
//! but not "every statement after it, unless hoisted".

use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;

pub struct NoUnreachable;

/// Whether a statement always leaves the block it is in.
fn exits(node: &tree_sitter::Node<'_>) -> bool {
    match node.kind() {
        "return_statement" | "throw_statement" | "break_statement" | "continue_statement" => true,
        "statement_block" => {
            let mut c = node.walk();
            node.named_children(&mut c).any(|s| exits(&s))
        }
        "if_statement" => {
            let consequence = node.child_by_field_name("consequence");
            let alternative = node.child_by_field_name("alternative").and_then(|a| a.named_child(0));
            matches!((consequence, alternative), (Some(c), Some(a)) if exits(&c) && exits(&a))
        }
        "try_statement" => {
            let finally = node.child_by_field_name("finalizer").and_then(|f| f.child_by_field_name("body").or(Some(f)));
            if finally.is_some_and(|f| exits(&f)) {
                return true;
            }
            let body = node.child_by_field_name("body");
            let handler = node.child_by_field_name("handler").and_then(|h| h.child_by_field_name("body"));
            body.is_some_and(|b| exits(&b)) && handler.is_none_or(|h| exits(&h))
        }
        _ => false,
    }
}

/// Hoisted declarations still take effect, so they are not unreachable.
fn hoisted(node: &tree_sitter::Node<'_>) -> bool {
    match node.kind() {
        "function_declaration" | "generator_function_declaration" | "empty_statement" | "comment" => true,
        "variable_declaration" => {
            let mut c = node.walk();
            node.named_children(&mut c).all(|d| d.kind() != "variable_declarator" || d.child_by_field_name("value").is_none())
        }
        _ => false,
    }
}

impl NativeRule for NoUnreachable {
    fn id(&self) -> &'static str {
        "js/no-unreachable"
    }

    fn languages(&self) -> &'static [Language] {
        &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx]
    }

    fn kinds(&self) -> &'static [&'static str] {
        &["program", "statement_block", "switch_case", "switch_default"]
    }

    fn check(&self, node: SyntaxNode<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let ts = node.get_inner_node();
        let mut c = ts.walk();
        let statements: Vec<_> = ts.named_children(&mut c).filter(|n| !n.is_extra()).collect();
        let body_start = if matches!(ts.kind(), "switch_case" | "switch_default") {
            // Skip the `case <value>` expression: only statements count.
            ts.child_by_field_name("value").map(|v| v.end_byte()).unwrap_or(0)
        } else {
            0
        };
        let mut exited = false;
        let mut run: Option<(usize, usize)> = None;
        let flush = |run: &mut Option<(usize, usize)>, out: &mut Vec<NativeFinding>| {
            if let Some((start, end)) = run.take() {
                out.push(NativeFinding {
                    start,
                    end,
                    message: "Unreachable code: it comes after a `return`, `throw`, `break` or `continue`".into(),
                    edits: vec![TextEdit::delete(start, end)],
                });
            }
        };
        for s in statements {
            if s.start_byte() < body_start {
                continue;
            }
            if exited {
                if hoisted(&s) {
                    flush(&mut run, out);
                } else {
                    run = Some(match run {
                        Some((start, _)) => (start, s.end_byte()),
                        None => (s.start_byte(), s.end_byte()),
                    });
                }
            } else if exits(&s) {
                exited = true;
            }
        }
        flush(&mut run, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(src: &str) -> Vec<String> {
        crate::run(&NoUnreachable, Language::Typescript, src).into_iter().map(|f| src[f.start..f.end].to_owned()).collect()
    }

    #[test]
    fn after_exits() {
        assert_eq!(found("function f() { return 1; a(); b(); }"), ["a(); b();"]);
        assert_eq!(found("function f() { throw e; function g() {} var v; c(); }"), ["c();"]);
        assert_eq!(found("for (;;) { if (x) { break; } else { continue; } y(); }"), ["y();"]);
        assert_eq!(found("switch (x) { case 1: return 1; z(); default: w(); }"), ["z();"]);
        assert_eq!(found("function f() { try { return 1; } finally { cleanup(); } after(); }"), ["after();"]);
    }

    #[test]
    fn reachable_code_is_left_alone() {
        assert!(found("function f() { if (x) return 1; return 2; }").is_empty());
        assert!(found("function f() { try { return 1; } catch (e) { log(e); } after(); }").is_empty());
        assert!(found("function f() { return; var hoisted; }").is_empty());
    }
}
