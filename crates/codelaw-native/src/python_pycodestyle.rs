//! Native rules behind `py/recommended`'s pycodestyle error ports (Ruff's
//! `E4`, `E7`): imports, comparisons, lambdas and ambiguous names. The
//! layout codes (`E1`–`E5`, `W`, `E701`–`E703`) belong to the formatter.

use crate::python_pyflakes::{Bind, bind_ctx};
use crate::python_util::{
    call_args, hit, hit_fix, indent_at, is_docstring_stmt, kids, names_one_of, py_rule, qualified_name, starts_line, statements, unparen,
};
use crate::util::{children, text};
use crate::{NativeContext, NativeFinding};
use codelaw_core::TextEdit;
use tree_sitter::Node;

fn multiple_imports_on_one_line(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    let names: Vec<Node> = kids(n).into_iter().filter(|c| matches!(c.kind(), "dotted_name" | "aliased_import")).collect();
    if names.len() < 2 {
        return;
    }
    let end = n.end_byte();
    let line_end = src[end..].find('\n').map(|i| end + i).unwrap_or(src.len());
    let alone = starts_line(src, n.start_byte()) && src[end..line_end].trim().is_empty() && !text(n, src).contains('\n');
    let edits = if alone {
        let indent = indent_at(src, n.start_byte());
        let lines: Vec<String> = names.iter().map(|x| format!("import {}", text(*x, src))).collect();
        vec![TextEdit::new(n.start_byte(), n.end_byte(), lines.join(&format!("\n{indent}")))]
    } else {
        Vec::new()
    };
    out.push(hit_fix(n, "Import one module per `import` statement", edits));
}
py_rule!(MultipleImportsOnOneLine, "py/multiple-imports-on-one-line", ["import_statement"], false, multiple_imports_on_one_line);

/// Statements that may come before a module's imports without ending the
/// import section.
fn allowed_before_imports(s: Node<'_>, ctx: &NativeContext<'_>) -> bool {
    let src = ctx.source;
    match s.kind() {
        "import_statement" | "import_from_statement" | "future_import_statement" | "if_statement" | "try_statement" | "with_statement" => true,
        "expression_statement" => {
            let Some(e) = s.named_child(0) else { return true };
            match e.kind() {
                "string" | "concatenated_string" => true,
                "assignment" | "augmented_assignment" => {
                    let left = e.child_by_field_name("left").map(|l| text(l, src)).unwrap_or("");
                    let dunder = left.starts_with("__") && left.ends_with("__") && !left.contains('.');
                    dunder || modifies(e.child_by_field_name("left"), ctx)
                }
                "call" => {
                    let f = e.child_by_field_name("function");
                    let Some(model) = ctx.semantic else { return false };
                    let Some(q) = f.and_then(|f| qualified_name(f, model, src)) else { return false };
                    q.starts_with("sys.path.") || q.starts_with("os.environ.") || q == "site.addsitedir" || q == "pytest.importorskip" || q == "matplotlib.use"
                }
                _ => false,
            }
        }
        "delete_statement" => kids(s).iter().all(|t| modifies(Some(*t), ctx)),
        _ => false,
    }
}

/// Whether an assignment target is `sys.path` or `os.environ` (or an item of them).
fn modifies(target: Option<Node<'_>>, ctx: &NativeContext<'_>) -> bool {
    let Some(mut t) = target else { return false };
    let Some(model) = ctx.semantic else { return false };
    if t.kind() == "subscript"
        && let Some(v) = t.child_by_field_name("value")
    {
        t = v;
    }
    names_one_of(t, model, ctx.source, &["sys.path", "os.environ"])
}

fn module_import_not_at_top(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let mut boundary = false;
    for (i, s) in statements(n).into_iter().enumerate() {
        let is_import = matches!(s.kind(), "import_statement" | "import_from_statement");
        if is_import && boundary {
            out.push(hit(s, "Module-level imports belong at the top of the file"));
            continue;
        }
        if i == 0 && is_docstring_stmt(s) {
            continue;
        }
        if !allowed_before_imports(s, ctx) {
            boundary = true;
        }
    }
}
py_rule!(ModuleImportNotAtTopOfFile, "py/module-import-not-at-top", ["module"], true, module_import_not_at_top);

/// A comparison's operands and operator tokens: `ops[i]` sits between
/// `operands[i]` and `operands[i + 1]`.
fn split_comparison<'t>(n: Node<'t>, src: &str) -> (Vec<Node<'t>>, Vec<(String, usize, usize)>) {
    let mut operands = Vec::new();
    let mut ops: Vec<(String, usize, usize)> = Vec::new();
    let mut pending: Option<(String, usize, usize)> = None;
    for c in children(n) {
        if c.kind() == "comment" {
            continue;
        }
        if c.is_named() {
            if let Some(p) = pending.take() {
                ops.push(p);
            }
            operands.push(c);
        } else {
            let t = text(c, src).split_whitespace().collect::<Vec<_>>().join(" ");
            pending = Some(match pending.take() {
                Some((pt, s, _)) => (format!("{pt} {t}"), s, c.end_byte()),
                None => (t, c.start_byte(), c.end_byte()),
            });
        }
    }
    (operands, ops)
}

fn none_comparison(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let (operands, ops) = split_comparison(n, ctx.source);
    for (i, (op, s, e)) in ops.iter().enumerate() {
        if op != "==" && op != "!=" {
            continue;
        }
        let (l, r) = (operands[i], operands[i + 1]);
        if unparen(l).kind() != "none" && unparen(r).kind() != "none" {
            continue;
        }
        let repl = if op == "==" { "is" } else { "is not" };
        out.push(NativeFinding {
            start: l.start_byte(),
            end: r.end_byte(),
            message: format!("Compare with `None` using `{repl}`, not `{op}`"),
            edits: vec![TextEdit::new(*s, *e, repl)],
        });
    }
}
py_rule!(NoneComparison, "py/none-comparison", ["comparison_operator"], false, none_comparison);

/// Wrap an expression in parentheses unless it is atomic.
fn atom(n: Node<'_>, src: &str) -> String {
    let t = text(n, src);
    match n.kind() {
        "identifier" | "attribute" | "call" | "subscript" | "parenthesized_expression" | "string" | "integer" | "float" | "list" | "tuple"
        | "dictionary" | "set" => t.to_owned(),
        _ => format!("({t})"),
    }
}

fn true_false_comparison(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    let (operands, ops) = split_comparison(n, src);
    for (i, (op, _, _)) in ops.iter().enumerate() {
        if op != "==" && op != "!=" {
            continue;
        }
        let (l, r) = (operands[i], operands[i + 1]);
        let (lit, other) = match (unparen(l).kind(), unparen(r).kind()) {
            ("true" | "false", _) => (unparen(l), r),
            (_, "true" | "false") => (unparen(r), l),
            _ => continue,
        };
        let truthy = (lit.kind() == "true") == (op == "==");
        let edits = if ops.len() == 1 && !matches!(unparen(other).kind(), "true" | "false") {
            let repl = if truthy { text(other, src).to_owned() } else { format!("not {}", atom(other, src)) };
            vec![TextEdit::new(n.start_byte(), n.end_byte(), repl)]
        } else {
            Vec::new()
        };
        let lit_t = text(lit, src);
        out.push(NativeFinding {
            start: l.start_byte(),
            end: r.end_byte(),
            message: format!("Don't compare with `{lit_t}` using `{op}`; test the value itself"),
            edits,
        });
    }
}
py_rule!(TrueFalseComparison, "py/true-false-comparison", ["comparison_operator"], false, true_false_comparison);

fn negated_test(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>, want: &str) {
    let src = ctx.source;
    let Some(arg) = n.child_by_field_name("argument") else { return };
    let inner = unparen(arg);
    if inner.kind() != "comparison_operator" {
        return;
    }
    let (operands, ops) = split_comparison(inner, src);
    if ops.len() != 1 || ops[0].0 != want {
        return;
    }
    let negated = if want == "in" { "not in" } else { "is not" };
    let repl = format!("{} {negated} {}", text(operands[0], src), text(operands[1], src));
    let msg = if want == "in" { "Write `x not in y`, not `not x in y`" } else { "Write `x is not y`, not `not x is y`" };
    out.push(hit_fix(inner, msg, vec![TextEdit::new(n.start_byte(), n.end_byte(), repl)]));
}
fn not_in_test(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    negated_test(n, ctx, out, "in")
}
fn not_is_test(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    negated_test(n, ctx, out, "is")
}
py_rule!(NotInTest, "py/not-in-test", ["not_operator"], false, not_in_test);
py_rule!(NotIsTest, "py/not-is-test", ["not_operator"], false, not_is_test);

fn is_type_call(n: Node<'_>, ctx: &NativeContext<'_>) -> bool {
    let n = unparen(n);
    if n.kind() != "call" {
        return false;
    }
    let Some(f) = n.child_by_field_name("function") else { return false };
    let Some(model) = ctx.semantic else { return false };
    let args = call_args(n, ctx.source);
    args.positional.len() == 1 && args.keywords.is_empty() && names_one_of(f, model, ctx.source, &["builtins.type"])
}

fn type_comparison(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let (operands, ops) = split_comparison(n, ctx.source);
    for (i, (op, _, _)) in ops.iter().enumerate() {
        if op != "==" && op != "!=" {
            continue;
        }
        let (l, r) = (operands[i], operands[i + 1]);
        if is_type_call(l, ctx) || is_type_call(r, ctx) {
            out.push(NativeFinding {
                start: l.start_byte(),
                end: r.end_byte(),
                message: "Compare types with `is` / `is not`, or test with `isinstance()`".into(),
                edits: Vec::new(),
            });
        }
    }
}
py_rule!(TypeComparison, "py/type-comparison", ["comparison_operator"], true, type_comparison);

fn bare_except(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if kids(n).first().is_some_and(|c| c.kind() != "block") {
        return;
    }
    let Some(kw) = n.child(0).filter(|c| c.kind() == "except") else { return };
    // A handler that re-raises doesn't swallow anything.
    if reraises(n) {
        return;
    }
    out.push(hit_fix(kw, "A bare `except:` also catches `KeyboardInterrupt` and `SystemExit`; name the exceptions", vec![TextEdit::new(kw.start_byte(), kw.end_byte(), "except Exception")]));
}
/// Whether a handler's body re-raises with a bare `raise` at its top level.
fn reraises(n: Node<'_>) -> bool {
    kids(n)
        .into_iter()
        .filter(|c| c.kind() == "block")
        .flat_map(statements)
        .any(|s| s.kind() == "raise_statement" && kids(s).is_empty())
}
py_rule!(BareExcept, "py/no-bare-except", ["except_clause"], false, bare_except);

fn lambda_assignment(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    let (Some(left), Some(right)) = (n.child_by_field_name("left"), n.child_by_field_name("right")) else { return };
    if left.kind() != "identifier" {
        return;
    }
    let lambda = unparen(right);
    if lambda.kind() != "lambda" {
        return;
    }
    let name = text(left, src);
    let stmt = n.parent().filter(|p| p.kind() == "expression_statement");
    let mut edits = Vec::new();
    if left.kind() == "identifier"
        && let Some(stmt) = stmt
        && starts_line(src, stmt.start_byte())
        && !text(stmt, src).contains('\n')
        && let Some(body) = lambda.child_by_field_name("body")
    {
        let end = stmt.end_byte();
        let line_end = src[end..].find('\n').map(|i| end + i).unwrap_or(src.len());
        if src[end..line_end].trim().is_empty() {
            let params = lambda.child_by_field_name("parameters").map(|p| text(p, src)).unwrap_or("");
            let indent = indent_at(src, stmt.start_byte());
            let def = format!("def {name}({params}):\n{indent}    return {}", text(body, src));
            edits.push(TextEdit::new(stmt.start_byte(), stmt.end_byte(), def));
        }
    }
    out.push(hit_fix(left, format!("Define `{name}` with `def` instead of assigning a lambda"), edits));
}
py_rule!(LambdaAssignment, "py/lambda-assignment", ["assignment"], false, lambda_assignment);

const AMBIGUOUS: &[&str] = &["l", "O", "I"];

fn ambiguous_variable_name(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let name = text(n, ctx.source);
    if !AMBIGUOUS.contains(&name) {
        return;
    }
    let Some(model) = ctx.semantic else { return };
    let augmented = n.parent().is_some_and(|p| p.kind() == "augmented_assignment" && p.child_by_field_name("left") == Some(n));
    if model.declared_at(n.start_byte()).is_none() && !augmented {
        return;
    }
    if matches!(bind_ctx(n).0, Bind::Def | Bind::Class) {
        return;
    }
    let mut p = n.parent();
    while let Some(x) = p {
        if x.kind() == "lambda_parameters" {
            return;
        }
        if matches!(x.kind(), "block" | "module") {
            break;
        }
        p = x.parent();
    }
    out.push(hit(n, format!("`{name}` is easily mistaken for `1` or `0`; use a longer name")));
}
py_rule!(AmbiguousVariableName, "py/ambiguous-variable-name", ["identifier"], true, ambiguous_variable_name);

fn ambiguous_def_name(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(name) = n.child_by_field_name("name") else { return };
    let t = text(name, ctx.source);
    if AMBIGUOUS.contains(&t) {
        let what = if n.kind() == "class_definition" { "class" } else { "function" };
        out.push(hit(name, format!("The {what} name `{t}` is easily mistaken for `1` or `0`; use a longer name")));
    }
}
py_rule!(AmbiguousClassName, "py/ambiguous-class-name", ["class_definition"], false, ambiguous_def_name);
py_rule!(AmbiguousFunctionName, "py/ambiguous-function-name", ["function_definition"], false, ambiguous_def_name);

#[cfg(test)]
mod tests {
    use crate::run;
    use codelaw_core::Language;

    fn fixed(rule: &dyn crate::NativeRule, src: &str) -> String {
        let mut s = src.to_owned();
        let mut f = run(rule, Language::Python, src);
        let mut edits: Vec<codelaw_core::TextEdit> = f.drain(..).flat_map(|x| x.edits).collect();
        edits.sort_by_key(|e| std::cmp::Reverse(e.start));
        for e in edits {
            s.replace_range(e.start..e.end, &e.text);
        }
        s
    }

    #[test]
    fn comparison_fixes() {
        assert_eq!(fixed(&super::NoneComparison, "if None != x: pass\n"), "if None is not x: pass\n");
        assert_eq!(fixed(&super::TrueFalseComparison, "if x == False: pass\n"), "if not x: pass\n");
        assert_eq!(fixed(&super::NotInTest, "if not (x in y): pass\n"), "if x not in y: pass\n");
        assert_eq!(fixed(&super::LambdaAssignment, "class C:\n    m = lambda self: 1\n"), "class C:\n    def m(self):\n        return 1\n");
    }

    #[test]
    fn import_boundary() {
        let src = "\"\"\"d\"\"\"\nimport os\nimport sys\nsys.path.insert(0, 'x')\nimport a\nos.environ['X'] = '1'\nimport b\nprint(1)\nimport c\n";
        assert_eq!(run(&super::ModuleImportNotAtTopOfFile, Language::Python, src).len(), 1);
    }
}
