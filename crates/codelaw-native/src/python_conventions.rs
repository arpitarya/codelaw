//! Native rules behind `py/conventions`, the starter catalogue's house
//! conventions that no upstream linter's preset covers.

use crate::python_util::{call_args, hit_fix, is_docstring_stmt, names_one_of, py_rule, statements};
use crate::util::text;
use crate::{NativeContext, NativeFinding};
use codelaw_core::TextEdit;
use codelaw_semantic::Imported;
use tree_sitter::Node;

/// Where `import logging` goes: after the docstring and `__future__` imports.
fn import_point(root: Node<'_>, src: &str) -> usize {
    let mut at = 0;
    for (i, s) in statements(root).into_iter().enumerate() {
        if (i == 0 && is_docstring_stmt(s)) || s.kind() == "future_import_statement" {
            let end = s.end_byte();
            at = src[end..].find('\n').map(|j| end + j + 1).unwrap_or(src.len());
        } else {
            break;
        }
    }
    at
}

fn no_print(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = ctx.semantic else { return };
    let src = ctx.source;
    let Some(f) = n.child_by_field_name("function") else { return };
    if !names_one_of(f, m, src, &["builtins.print"]) {
        return;
    }
    let args = call_args(n, src);
    let mut edits = Vec::new();
    if args.positional.len() == 1 && args.keywords.is_empty() && !args.starred {
        edits.push(TextEdit::new(n.start_byte(), n.end_byte(), format!("logging.getLogger(__name__).info(\"%s\", {})", text(args.positional[0], src))));
        let has_logging = m.imports.iter().any(|i| i.source == "logging" && i.imported == Imported::Module && i.local.is_some_and(|s| m.symbol(s).name == "logging"));
        if !has_logging {
            let mut root = n;
            while let Some(p) = root.parent() {
                root = p;
            }
            let at = import_point(root, src);
            edits.push(TextEdit::new(at, at, "import logging\n"));
            edits.sort_by_key(|e| e.start);
        }
    }
    out.push(hit_fix(n, "Report through `logging`, not `print()`, so callers control where output goes", edits));
}
py_rule!(NoPrint, "py/no-print", ["call"], true, no_print);
