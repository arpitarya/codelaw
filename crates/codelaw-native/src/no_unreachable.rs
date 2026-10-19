//! `js/no-unreachable`: statements after a `return`, `throw`, `break` or
//! `continue` in the same block never run. A pattern can match the exit,
//! but not "every statement after it, unless hoisted".

use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use crate::util::{exits, whole_lines};

pub struct NoUnreachable;

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

    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let src = ctx.source;
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
                let (a, b) = whole_lines(src, start, end);
                out.push(NativeFinding {
                    start,
                    end,
                    message: "Unreachable code: it comes after a `return`, `throw`, `break` or `continue`".into(),
                    edits: vec![TextEdit::delete(a, b)],
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
