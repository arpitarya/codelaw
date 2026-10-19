//! `js/no-useless-assignment`: a value stored in a variable that nothing
//! reads. Two cases a straight-line reading can prove:
//!
//! - **overwritten:** a value assigned, then assigned again by a statement
//!   in the same block before anything reads it;
//! - **never read:** an assignment after which no code reads the variable,
//!   outside any loop.
//!
//! A variable a nested function touches is skipped: its reads can happen at
//! any time.

use crate::util::is_function;
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use codelaw_semantic::{ScopeKind, SymbolKind};
use tree_sitter::Node;

pub struct NoUselessAssignment;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    /// A store; `at` is when it takes effect (after its right side runs).
    Write { at: usize, start: usize, end: usize },
    Read { at: usize },
}

impl Event {
    fn at(&self) -> usize {
        match self {
            Event::Write { at, .. } | Event::Read { at } => *at,
        }
    }
}

fn ancestors<'t>(n: Node<'t>) -> impl Iterator<Item = Node<'t>> {
    std::iter::successors(n.parent(), |p| p.parent())
}

fn uses_arguments(function: Node<'_>, src: &str) -> bool {
    let mut stack = vec![function];
    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" && &src[n.byte_range()] == "arguments" {
            return true;
        }
        let mut c = n.walk();
        stack.extend(n.children(&mut c).filter(|x| !(is_function(x.kind()) && x.id() != function.id())));
    }
    false
}

fn block_of(n: Node<'_>) -> Option<Node<'_>> {
    ancestors(n).find(|a| matches!(a.kind(), "statement_block" | "program"))
}

const LOOPS: &[&str] = &["for_statement", "for_in_statement", "while_statement", "do_statement"];

/// The function-like node a node belongs to (or the program).
fn owner(n: Node<'_>) -> Option<Node<'_>> {
    ancestors(n).find(|a| is_function(a.kind()) || a.kind() == "arrow_function" || a.kind() == "program")
}

/// The statement a write sits in, if that statement is a plain assignment
/// `x = …;` directly in a block.
fn assignment_statement(ident: Node<'_>) -> Option<Node<'_>> {
    let assign = ident.parent().filter(|p| p.kind() == "assignment_expression")?;
    let stmt = assign.parent().filter(|p| p.kind() == "expression_statement")?;
    stmt.parent().filter(|b| matches!(b.kind(), "statement_block" | "program"))?;
    Some(stmt)
}

impl NativeRule for NoUselessAssignment {
    fn id(&self) -> &'static str {
        "js/no-useless-assignment"
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
        for s in &model.symbols {
            if !matches!(s.kind, SymbolKind::Let | SymbolKind::Var | SymbolKind::Param) || s.exported || s.refs.is_empty() {
                continue;
            }
            if model.scopes[s.scope].kind == ScopeKind::Module {
                continue;
            }
            let Some(&(ds, de)) = s.decls.first() else { continue };
            let Some(decl) = root.descendant_for_byte_range(ds, de) else { continue };
            let home = owner(decl);
            // In sloppy code `arguments` aliases the parameters, so a store to
            // one can be read through it.
            if s.kind == SymbolKind::Param && home.is_some_and(|h| uses_arguments(h, ctx.source)) {
                continue;
            }
            let mut events = Vec::new();
            let mut captured = false;
            // The declaration's initialiser is the first store.
            if let Some(d) = decl.parent().filter(|p| p.kind() == "variable_declarator" && p.child_by_field_name("value").is_some()) {
                events.push(Event::Write { at: d.end_byte(), start: ds, end: de });
            }
            for &r in &s.refs {
                let reference = &model.references[r];
                let Some(n) = root.descendant_for_byte_range(reference.start, reference.end) else { continue };
                if owner(n) != home {
                    captured = true;
                    break;
                }
                if reference.read {
                    events.push(Event::Read { at: reference.start });
                }
                if reference.write {
                    let at = n.parent().filter(|p| p.kind() == "assignment_expression").map(|p| p.end_byte()).unwrap_or(reference.end);
                    events.push(Event::Write { at, start: reference.start, end: reference.end });
                }
            }
            if captured {
                continue;
            }
            events.sort_by_key(|e| e.at());
            for (i, e) in events.iter().enumerate() {
                let Event::Write { start, end, .. } = *e else { continue };
                let Some(ident) = root.descendant_for_byte_range(start, end) else { continue };
                let in_loop = ancestors(ident).take_while(|a| Some(*a) != home).any(|a| LOOPS.contains(&a.kind()));
                let in_try = ancestors(ident).take_while(|a| Some(*a) != home).any(|a| matches!(a.kind(), "try_statement" | "catch_clause"));
                if in_loop || in_try {
                    continue;
                }
                let next = events.get(i + 1);
                // An initialiser nothing reads is `no-unused-vars`'s to report.
                let never_read = next.is_none() && start != ds;
                let overwritten = match next {
                    Some(Event::Write { start: ns, end: ne, .. }) => {
                        let Some(nid) = root.descendant_for_byte_range(*ns, *ne) else { continue };
                        // The overwrite must be unconditional: a plain statement in
                        // the same block as this store.
                        assignment_statement(nid).is_some() && block_of(nid) == block_of(ident)
                    }
                    _ => false,
                };
                if overwritten || never_read {
                    let why = if overwritten { "is overwritten before anything reads it" } else { "is never read" };
                    out.push(NativeFinding { start, end, message: format!("The value assigned to `{}` here {why}", s.name), edits: Vec::new() });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(src: &str) -> Vec<String> {
        crate::run(&NoUselessAssignment, Language::Javascript, src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn dead_stores() {
        assert_eq!(found("function f() {\n  let x = 0;\n  x = compute();\n  return x;\n}"), ["The value assigned to `x` here is overwritten before anything reads it"]);
        assert_eq!(found("function f(a) {\n  let y = a;\n  use(y);\n  y = 2;\n}"), ["The value assigned to `y` here is never read"]);
    }

    #[test]
    fn live_stores() {
        assert!(found("function f(c) {\n  let x = 0;\n  if (c) { x = 1; }\n  return x;\n}").is_empty());
        assert!(found("function f() {\n  let x = 0;\n  x = x + 1;\n  return x;\n}").is_empty());
        assert!(found("function f(items) {\n  let last;\n  for (const i of items) { last = i; }\n  return last;\n}").is_empty());
        assert!(found("function f() {\n  let x = 1;\n  const g = () => x;\n  x = 2;\n  return g;\n}").is_empty());
        assert!(found("function f() {\n  let n = 0;\n  while (n < 3) { n = n + 1; }\n  return n;\n}").is_empty());
        assert!(found("function f(er) {\n  if (ok(er)) er = null;\n  cb.apply(this, arguments);\n}").is_empty());
    }
}
