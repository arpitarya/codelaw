//! Native rules behind `py/recommended`'s flake8-bugbear ports (Ruff's
//! `B` rules): likely bugs and design problems a pattern can't see.

use crate::python_pyflakes::different_forks;
use crate::python_util::{
    call_args, decorator_target, decorators, hit, hit_fix, indent_at, is_docstring_stmt, is_empty_body, is_fstring, is_pure, is_string,
    kids, names_one_of, py_rule, qualified_name, starts_line, statements, string_value, unparen,
};
use crate::util::{children, text};
use crate::{NativeContext, NativeFinding};
use codelaw_core::TextEdit;
use codelaw_semantic::{ScopeKind, SemanticModel};
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::Node;

fn model<'a>(ctx: &'a NativeContext<'_>) -> Option<&'a SemanticModel> {
    ctx.semantic
}

fn unary_prefix_increment(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let (Some(op), Some(arg)) = (n.child_by_field_name("operator"), n.child_by_field_name("argument")) else { return };
    let op = text(op, ctx.source);
    if (op == "+" || op == "-")
        && arg.kind() == "unary_operator"
        && arg.child_by_field_name("operator").is_some_and(|o| text(o, ctx.source) == op)
        && n.parent().is_none_or(|p| p.kind() != "unary_operator" || p.child_by_field_name("operator").is_none_or(|o| text(o, ctx.source) != op))
    {
        let what = if op == "+" { "increment" } else { "decrement" };
        out.push(hit(n, format!("Python has no `{op}{op}` {what} operator; this applies unary `{op}` twice")));
    }
}
py_rule!(UnaryPrefixIncrementDecrement, "py/unary-prefix-increment", ["unary_operator"], false, unary_prefix_increment);

fn assignment_to_os_environ(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = model(ctx) else { return };
    let Some(left) = n.child_by_field_name("left") else { return };
    if left.kind() == "attribute" && names_one_of(left, m, ctx.source, &["os.environ"]) {
        out.push(hit(left, "Assigning to `os.environ` replaces the mapping but doesn't change the process environment"));
    }
}
py_rule!(AssignmentToOsEnviron, "py/assignment-to-os-environ", ["assignment"], true, assignment_to_os_environ);

fn builtin_call<'t>(n: Node<'t>, ctx: &NativeContext<'_>, names: &[&str]) -> Option<String> {
    let f = n.child_by_field_name("function")?;
    let q = qualified_name(f, model(ctx)?, ctx.source)?;
    names.iter().find(|x| q == format!("builtins.{x}")).map(|x| x.to_string())
}

fn unreliable_callable_check(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(name) = builtin_call(n, ctx, &["hasattr", "getattr"]) else { return };
    let args = call_args(n, ctx.source);
    if args.positional.len() < 2 || string_value(args.positional[1], ctx.source).as_deref() != Some("__call__") {
        return;
    }
    let edits = if name == "hasattr" && args.positional.len() == 2 && args.keywords.is_empty() && !args.starred {
        vec![TextEdit::new(n.start_byte(), n.end_byte(), format!("callable({})", text(args.positional[0], ctx.source)))]
    } else {
        Vec::new()
    };
    out.push(hit_fix(n, "Test for a callable with `callable(x)`; checking for `__call__` is unreliable", edits));
}
py_rule!(UnreliableCallableCheck, "py/unreliable-callable-check", ["call"], true, unreliable_callable_check);

fn strip_with_multi_characters(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(f) = n.child_by_field_name("function").filter(|f| f.kind() == "attribute") else { return };
    let method = f.child_by_field_name("attribute").map(|a| text(a, ctx.source)).unwrap_or("");
    if !matches!(method, "strip" | "lstrip" | "rstrip") {
        return;
    }
    let args = call_args(n, ctx.source);
    if args.positional.len() != 1 || !args.keywords.is_empty() {
        return;
    }
    let Some(v) = string_value(args.positional[0], ctx.source) else { return };
    let chars: Vec<char> = v.chars().collect();
    let unique: BTreeSet<char> = chars.iter().copied().collect();
    if chars.len() > 1 && unique.len() != chars.len() {
        out.push(hit(n, format!("`.{method}()` removes a set of characters, not a prefix or suffix; the repeated characters suggest a string was meant")));
    }
}
py_rule!(StripWithMultiCharacters, "py/strip-with-multi-characters", ["call"], false, strip_with_multi_characters);

const MUTABLE_CALLS: &[&str] = &[
    "builtins.list", "builtins.dict", "builtins.set", "builtins.bytearray", "collections.Counter", "collections.OrderedDict",
    "collections.defaultdict", "collections.deque",
];

const IMMUTABLE_CALLS: &[&str] = &[
    "builtins.tuple", "builtins.frozenset", "builtins.bool", "builtins.bytes", "builtins.complex", "builtins.float", "builtins.int",
    "builtins.str", "builtins.range", "datetime.date", "datetime.datetime", "datetime.timedelta", "datetime.time", "decimal.Decimal",
    "fractions.Fraction", "operator.attrgetter", "operator.itemgetter", "operator.methodcaller", "pathlib.Path", "types.MappingProxyType",
    "re.compile",
];

fn is_mutable(n: Node<'_>, ctx: &NativeContext<'_>) -> bool {
    let n = unparen(n);
    match n.kind() {
        "list" | "dictionary" | "set" | "list_comprehension" | "dictionary_comprehension" | "set_comprehension" => true,
        "call" => model(ctx).is_some_and(|m| n.child_by_field_name("function").is_some_and(|f| names_one_of(f, m, ctx.source, MUTABLE_CALLS))),
        _ => false,
    }
}

fn extra_immutable(ctx: &NativeContext<'_>) -> Vec<String> {
    ctx.options.get("extend_immutable_calls").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|v| v.as_str().map(str::to_owned)).collect()
}

/// Calls in a default value, outside lambdas, that are neither mutable
/// constructors nor known to return immutable values.
fn suspicious_calls<'t>(n: Node<'t>, ctx: &NativeContext<'_>, extra: &[String], out: &mut Vec<Node<'t>>) {
    match n.kind() {
        "lambda" => {}
        "call" => {
            let Some(m) = model(ctx) else { return };
            let f = n.child_by_field_name("function");
            let q = f.and_then(|f| qualified_name(f, m, ctx.source));
            let shown = f.map(|f| text(f, ctx.source).to_owned()).unwrap_or_default();
            let immutable = q.as_deref().is_some_and(|q| IMMUTABLE_CALLS.contains(&q))
                || extra.iter().any(|e| q.as_deref() == Some(e.as_str()) || *e == shown);
            if is_mutable(n, ctx) || immutable {
                for c in kids(n) {
                    suspicious_calls(c, ctx, extra, out);
                }
            } else {
                out.push(n);
            }
        }
        _ => {
            for c in kids(n) {
                suspicious_calls(c, ctx, extra, out);
            }
        }
    }
}

fn defaults<'t>(def: Node<'t>) -> Vec<(Node<'t>, Node<'t>, Node<'t>)> {
    let mut out = Vec::new();
    if let Some(params) = def.child_by_field_name("parameters") {
        for p in kids(params) {
            if matches!(p.kind(), "default_parameter" | "typed_default_parameter")
                && let (Some(name), Some(value)) = (p.child_by_field_name("name"), p.child_by_field_name("value"))
            {
                out.push((p, name, value));
            }
        }
    }
    out
}

/// Where to insert statements at the start of a function body: after its
/// docstring, at the start of a line.
fn body_insertion(def: Node<'_>, src: &str) -> Option<(usize, String)> {
    let body = def.child_by_field_name("body")?;
    let stmts = statements(body);
    let first = *stmts.first()?;
    if !starts_line(src, first.start_byte()) {
        return None;
    }
    let indent = indent_at(src, first.start_byte());
    let anchor = if is_docstring_stmt(first) {
        match stmts.get(1) {
            Some(s) => *s,
            None => {
                let end = first.end_byte();
                let line_end = src[end..].find('\n').map(|i| end + i + 1)?;
                return Some((line_end, indent));
            }
        }
    } else {
        first
    };
    let line_start = src[..anchor.start_byte()].rfind('\n').map(|i| i + 1).unwrap_or(0);
    Some((line_start, indent))
}

fn mutable_argument_default(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    for (p, name, value) in defaults(n) {
        if !is_mutable(value, ctx) {
            continue;
        }
        let mut edits = Vec::new();
        let mut names = Vec::new();
        loaded_names(value, &mut names);
        let self_reference = names.iter().any(|x| text(*x, src) == text(name, src));
        if p.kind() == "default_parameter"
            && !self_reference
            && !text(value, src).contains('\n')
            && let Some((at, indent)) = body_insertion(n, src)
        {
            let nm = text(name, src);
            edits.push(TextEdit::new(value.start_byte(), value.end_byte(), "None"));
            edits.push(TextEdit::new(at, at, format!("{indent}if {nm} is None:\n{indent}    {nm} = {}\n", text(value, src))));
        }
        out.push(hit_fix(value, "A mutable default is created once and shared by every call; default to `None` and create it in the body", edits));
    }
}
py_rule!(MutableArgumentDefault, "py/no-mutable-default", ["function_definition"], true, mutable_argument_default);

fn function_call_in_default_argument(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let extra = extra_immutable(ctx);
    for (_, _, value) in defaults(n) {
        let mut calls = Vec::new();
        suspicious_calls(value, ctx, &extra, &mut calls);
        for c in calls {
            let f = c.child_by_field_name("function").map(|f| text(f, ctx.source)).unwrap_or("");
            out.push(hit(c, format!("`{f}()` runs once, when the function is defined, not on every call")));
        }
    }
}
py_rule!(FunctionCallInDefaultArgument, "py/function-call-in-default-argument", ["function_definition"], true, function_call_in_default_argument);

fn mutable_contextvar_default(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = model(ctx) else { return };
    let Some(f) = n.child_by_field_name("function") else { return };
    if !names_one_of(f, m, ctx.source, &["contextvars.ContextVar"]) {
        return;
    }
    let Some(default) = call_args(n, ctx.source).keyword("default") else { return };
    let mut calls = Vec::new();
    suspicious_calls(default, ctx, &extra_immutable(ctx), &mut calls);
    if is_mutable(default, ctx) || !calls.is_empty() {
        out.push(hit(default, "A `ContextVar` default is shared by every context; don't make it mutable or computed"));
    }
}
py_rule!(MutableContextvarDefault, "py/mutable-contextvar-default", ["call"], true, mutable_contextvar_default);

fn target_names<'t>(n: Node<'t>, out: &mut Vec<Node<'t>>) {
    match n.kind() {
        "identifier" => out.push(n),
        "pattern_list" | "tuple_pattern" | "list_pattern" | "tuple" | "list" | "parenthesized_expression" | "list_splat_pattern"
        | "list_splat" => {
            for c in kids(n) {
                target_names(c, out);
            }
        }
        _ => {}
    }
}

fn unused_loop_control_variable(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = model(ctx) else { return };
    let (Some(left), Some(body)) = (n.child_by_field_name("left"), n.child_by_field_name("body")) else { return };
    let mut names = Vec::new();
    target_names(left, &mut names);
    for id in names {
        let name = text(id, ctx.source);
        if name.starts_with('_') {
            continue;
        }
        let Some(sym) = m.declared_at(id.start_byte()) else { continue };
        let s = m.symbol(sym);
        let refs: Vec<_> = s.refs.iter().map(|&r| &m.references[r]).filter(|r| r.read).collect();
        if refs.iter().any(|r| body.start_byte() <= r.start && r.start < body.end_byte()) {
            continue;
        }
        let edits = if refs.is_empty() && s.decls.len() == 1 {
            vec![TextEdit::new(id.start_byte(), id.end_byte(), format!("_{name}"))]
        } else {
            Vec::new()
        };
        out.push(hit_fix(id, format!("The loop variable `{name}` is not used in the loop body; name it `_{name}`"), edits));
    }
}
py_rule!(UnusedLoopControlVariable, "py/unused-loop-control-variable", ["for_statement"], true, unused_loop_control_variable);

const KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif", "else", "except",
    "finally", "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "while", "with", "yield",
];

fn plain_attribute_name(n: Node<'_>, src: &str) -> Option<String> {
    if n.kind() != "string" || is_fstring(n, src) {
        return None;
    }
    let v = string_value(n, src)?;
    let ident = v.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_') && v.chars().all(|c| c.is_alphanumeric() || c == '_');
    let mangled = v.starts_with("__") && !v.ends_with("__");
    (ident && !mangled && !KEYWORDS.contains(&v.as_str()) && v.is_ascii()).then_some(v)
}

fn as_object(n: Node<'_>, src: &str) -> String {
    let t = text(n, src);
    match n.kind() {
        "identifier" | "attribute" | "call" | "subscript" | "parenthesized_expression" | "string" | "list" | "dictionary" | "tuple" => {
            t.to_owned()
        }
        _ => format!("({t})"),
    }
}

fn getattr_with_constant(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if builtin_call(n, ctx, &["getattr"]).is_none() {
        return;
    }
    let src = ctx.source;
    let args = call_args(n, src);
    if args.positional.len() != 2 || !args.keywords.is_empty() || args.starred {
        return;
    }
    let Some(attr) = plain_attribute_name(args.positional[1], src) else { return };
    let repl = format!("{}.{attr}", as_object(args.positional[0], src));
    out.push(hit_fix(n, format!("Write `{repl}`; `getattr` with a constant name is no safer"), vec![TextEdit::new(n.start_byte(), n.end_byte(), repl)]));
}
py_rule!(GetAttrWithConstant, "py/getattr-with-constant", ["call"], true, getattr_with_constant);

fn setattr_with_constant(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if builtin_call(n, ctx, &["setattr"]).is_none() || n.parent().is_none_or(|p| p.kind() != "expression_statement") {
        return;
    }
    let src = ctx.source;
    let args = call_args(n, src);
    if args.positional.len() != 3 || !args.keywords.is_empty() || args.starred {
        return;
    }
    let Some(attr) = plain_attribute_name(args.positional[1], src) else { return };
    let repl = format!("{}.{attr} = {}", as_object(args.positional[0], src), text(args.positional[2], src));
    out.push(hit_fix(n, format!("Write `{repl}`; `setattr` with a constant name is no safer"), vec![TextEdit::new(n.start_byte(), n.end_byte(), repl)]));
}
py_rule!(SetAttrWithConstant, "py/setattr-with-constant", ["call"], true, setattr_with_constant);

fn assert_false(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let items = kids(n);
    let Some(first) = items.first().filter(|f| f.kind() == "false") else { return };
    let msg = items.get(1).map(|m| text(*m, ctx.source)).unwrap_or("");
    let edits = vec![TextEdit::new(n.start_byte(), n.end_byte(), format!("raise AssertionError({msg})"))];
    out.push(hit_fix(*first, "`python -O` removes `assert False`; raise `AssertionError` instead", edits));
}
py_rule!(AssertFalse, "py/assert-false", ["assert_statement"], false, assert_false);

fn jump_statement_in_finally(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let is_return = n.kind() == "return_statement";
    let mut cur = n;
    while let Some(p) = cur.parent() {
        match p.kind() {
            "function_definition" | "lambda" | "class_definition" => return,
            "for_statement" | "while_statement" if !is_return && p.child_by_field_name("body") == Some(cur) => return,
            "finally_clause" => {
                let kw = text(n, ctx.source).split_whitespace().next().unwrap_or("return");
                out.push(hit(n, format!("`{kw}` in a `finally` block silently discards any exception being raised")));
                return;
            }
            _ => {}
        }
        cur = p;
    }
}
py_rule!(JumpStatementInFinally, "py/jump-statement-in-finally", ["return_statement", "break_statement", "continue_statement"], false, jump_statement_in_finally);

/// The exception expression of an `except` clause (without `as`).
fn handler_type(h: Node<'_>) -> Option<Node<'_>> {
    let first = *kids(h).first()?;
    match first.kind() {
        "block" => None,
        "as_pattern" => first.named_child(0),
        _ => Some(first),
    }
}

fn redundant_tuple_in_exception_handler(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(t) = handler_type(n) else { return };
    if t.kind() != "tuple" {
        return;
    }
    let items = kids(t);
    if let [only] = items[..]
        && only.kind() != "list_splat"
    {
        out.push(hit_fix(t, "A one-element tuple in `except` is redundant", vec![TextEdit::new(t.start_byte(), t.end_byte(), text(only, ctx.source))]));
    }
}
py_rule!(RedundantTupleInExceptionHandler, "py/redundant-tuple-in-exception-handler", ["except_clause"], false, redundant_tuple_in_exception_handler);

fn norm(n: Node<'_>, src: &str) -> String {
    text(n, src).split_whitespace().collect()
}

fn duplicate_handler_exception(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    let Some(t) = handler_type(n).filter(|t| t.kind() == "tuple") else { return };
    let items = kids(t);
    if items.iter().any(|i| i.kind() == "list_splat") {
        return;
    }
    let mut seen: Vec<String> = Vec::new();
    let mut keep: Vec<&str> = Vec::new();
    let mut dups = Vec::new();
    for i in &items {
        let k = norm(*i, src);
        if seen.contains(&k) {
            dups.push(text(*i, src).to_owned());
        } else {
            seen.push(k);
            keep.push(text(*i, src));
        }
    }
    if dups.is_empty() {
        return;
    }
    let repl = if keep.len() == 1 { keep[0].to_owned() } else { format!("({})", keep.join(", ")) };
    out.push(hit_fix(t, format!("`{}` is listed more than once in this handler", dups[0]), vec![TextEdit::new(t.start_byte(), t.end_byte(), repl)]));
}
py_rule!(DuplicateHandlerException, "py/duplicate-handler-exception", ["except_clause"], false, duplicate_handler_exception);

fn useless_comparison(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if let Some(e) = n.named_child(0)
        && n.named_child_count() == 1
        && e.kind() == "comparison_operator"
    {
        out.push(hit(e, "This comparison's result is thrown away; did you mean `=` or `assert`?"));
    }
}
py_rule!(UselessComparison, "py/useless-comparison", ["expression_statement"], false, useless_comparison);

fn raise_literal(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if let Some(e) = kids(n).first().map(|e| unparen(*e))
        && matches!(e.kind(), "string" | "concatenated_string" | "integer" | "float" | "true" | "false")
    {
        out.push(hit(e, "Only exceptions can be raised; raising a literal is a `TypeError`"));
    }
}
py_rule!(RaiseLiteral, "py/raise-literal", ["raise_statement"], false, raise_literal);

fn assert_raises_exception(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = model(ctx) else { return };
    let src = ctx.source;
    let Some(v) = n.child_by_field_name("value") else { return };
    let call = if v.kind() == "as_pattern" { v.named_child(0) } else { Some(v) };
    let Some(call) = call.filter(|c| c.kind() == "call") else { return };
    let Some(f) = call.child_by_field_name("function") else { return };
    let ft = text(f, src);
    let args = call_args(call, src);
    let Some(first) = args.positional.first() else { return };
    let blind = names_one_of(*first, m, src, &["builtins.Exception", "builtins.BaseException"]);
    if !blind {
        return;
    }
    let is_unittest = f.kind() == "attribute" && f.child_by_field_name("attribute").is_some_and(|a| text(a, src) == "assertRaises");
    let is_pytest = ft == "pytest.raises" || qualified_name(f, m, src).as_deref() == Some("pytest.raises");
    if (is_unittest && args.positional.len() == 1) || (is_pytest && args.keyword("match").is_none()) {
        out.push(hit(call, format!("`{}` passes for any error at all; assert the specific exception", text(*first, src))));
    }
}
py_rule!(AssertRaisesException, "py/assert-raises-exception", ["with_item"], true, assert_raises_exception);

fn attribute_chain(n: Node<'_>) -> bool {
    match n.kind() {
        "identifier" => true,
        "attribute" => n.child_by_field_name("object").is_some_and(attribute_chain),
        _ => false,
    }
}

fn useless_expression(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let parts = kids(n);
    if parts.len() > 1 {
        // `a.b, c.d` on its own: a tuple statement.
        if parts.iter().all(|c| is_pure(*c) || attribute_chain(*c)) {
            out.push(NativeFinding { start: n.start_byte(), end: n.end_byte(), message: "This expression does nothing; assign it, use it, or remove it".into(), edits: Vec::new() });
        }
        return;
    }
    let Some(e) = parts.first().copied() else { return };
    let inner = unparen(e);
    let flagged = match inner.kind() {
        "string" | "concatenated_string" | "ellipsis" | "comparison_operator" => false,
        "attribute" => attribute_chain(inner),
        "tuple" | "expression_list" => kids(inner).iter().all(|c| is_pure(*c) || attribute_chain(*c)),
        "binary_operator" => {
            let lit = |x: Option<Node<'_>>| x.is_some_and(|x| matches!(unparen(x).kind(), "integer" | "float"));
            lit(inner.child_by_field_name("left")) && lit(inner.child_by_field_name("right"))
        }
        _ => is_pure(inner),
    };
    if flagged {
        out.push(hit(e, "This expression does nothing; assign it, use it, or remove it"));
    }
}
py_rule!(UselessExpression, "py/useless-expression", ["expression_statement"], false, useless_expression);

/// The class a function is defined directly in, if it is a method.
fn method_class(def: Node<'_>) -> Option<Node<'_>> {
    let mut p = def.parent()?;
    if p.kind() == "decorated_definition" {
        p = p.parent()?;
    }
    if p.kind() != "block" {
        return None;
    }
    p.parent().filter(|c| c.kind() == "class_definition")
}

fn cached_instance_method(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = model(ctx) else { return };
    if method_class(n).is_none() {
        return;
    }
    let decs = decorators(n);
    let static_or_class = decs.iter().any(|d| names_one_of(*d, m, ctx.source, &["builtins.staticmethod", "builtins.classmethod"]));
    if static_or_class {
        return;
    }
    for d in decs {
        if names_one_of(decorator_target(d), m, ctx.source, &["functools.lru_cache", "functools.cache"]) {
            out.push(hit(d, "Caching a method keeps every `self` alive in the cache; cache a function or a `staticmethod` instead"));
        }
    }
}
py_rule!(CachedInstanceMethod, "py/cached-instance-method", ["function_definition"], true, cached_instance_method);

/// Identifiers loaded in an expression, not entering nested scopes.
fn loaded_names<'t>(n: Node<'t>, out: &mut Vec<Node<'t>>) {
    match n.kind() {
        "identifier" => out.push(n),
        "lambda" | "list_comprehension" | "set_comprehension" | "dictionary_comprehension" | "generator_expression" => {}
        "attribute" => {
            if let Some(o) = n.child_by_field_name("object") {
                loaded_names(o, out);
            }
        }
        "keyword_argument" => {
            if let Some(v) = n.child_by_field_name("value") {
                loaded_names(v, out);
            }
        }
        _ => {
            for c in kids(n) {
                loaded_names(c, out);
            }
        }
    }
}

fn loop_variable_overrides_iterator(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let (Some(left), Some(right)) = (n.child_by_field_name("left"), n.child_by_field_name("right")) else { return };
    let mut targets = Vec::new();
    target_names(left, &mut targets);
    let mut used = Vec::new();
    loaded_names(right, &mut used);
    let used: BTreeSet<&str> = used.iter().map(|u| text(*u, ctx.source)).collect();
    for t in targets {
        let name = text(t, ctx.source);
        if used.contains(name) {
            out.push(hit(t, format!("The loop variable `{name}` replaces the value being iterated over")));
        }
    }
}
py_rule!(LoopVariableOverridesIterator, "py/loop-variable-overrides-iterator", ["for_statement"], false, loop_variable_overrides_iterator);

fn fstring_docstring(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let body = if n.kind() == "module" { Some(n) } else { n.child_by_field_name("body") };
    let Some(body) = body else { return };
    let Some(first) = statements(body).first().copied() else { return };
    if first.kind() == "expression_statement"
        && let Some(s) = first.named_child(0)
        && is_string(s)
        && is_fstring(s, ctx.source)
    {
        out.push(hit(s, "An f-string is not a docstring; Python leaves `__doc__` empty"));
    }
}
py_rule!(FStringDocstring, "py/f-string-docstring", ["module", "function_definition", "class_definition"], false, fstring_docstring);

fn useless_contextlib_suppress(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = model(ctx) else { return };
    let Some(v) = n.child_by_field_name("value") else { return };
    let call = if v.kind() == "as_pattern" { v.named_child(0) } else { Some(v) };
    let Some(call) = call.filter(|c| c.kind() == "call") else { return };
    if call.child_by_field_name("function").is_some_and(|f| names_one_of(f, m, ctx.source, &["contextlib.suppress"])) {
        let a = call_args(call, ctx.source);
        if a.positional.is_empty() && a.keywords.is_empty() && !a.starred {
            out.push(hit(call, "`contextlib.suppress()` with no exceptions suppresses nothing"));
        }
    }
}
py_rule!(UselessContextlibSuppress, "py/useless-contextlib-suppress", ["with_item"], true, useless_contextlib_suppress);

/// Names a loop (re)binds on each iteration, outside nested scopes.
fn loop_bound_names(n: Node<'_>, src: &str, out: &mut BTreeSet<String>) {
    match n.kind() {
        "function_definition" | "class_definition" | "lambda" => {
            if let Some(name) = n.child_by_field_name("name") {
                out.insert(text(name, src).to_owned());
            }
        }
        "assignment" | "augmented_assignment" | "for_statement" | "for_in_clause" => {
            if let Some(l) = n.child_by_field_name("left") {
                let mut names = Vec::new();
                target_names(l, &mut names);
                out.extend(names.iter().map(|x| text(*x, src).to_owned()));
            }
            for c in kids(n) {
                loop_bound_names(c, src, out);
            }
        }
        "named_expression" => {
            if let Some(name) = n.child_by_field_name("name") {
                out.insert(text(name, src).to_owned());
            }
            for c in kids(n) {
                loop_bound_names(c, src, out);
            }
        }
        _ => {
            for c in kids(n) {
                loop_bound_names(c, src, out);
            }
        }
    }
}

/// Functions and lambdas inside a loop body, not inside other functions.
fn nested_functions<'t>(n: Node<'t>, out: &mut Vec<Node<'t>>) {
    for c in kids(n) {
        match c.kind() {
            "function_definition" | "lambda" => out.push(c),
            "class_definition" => {}
            _ => nested_functions(c, out),
        }
    }
}

/// Whether a function or lambda is used right away, so it sees the loop
/// variable's current value: called on the spot, or handed to `map`,
/// `filter`, `reduce` or a `key=` argument.
fn used_immediately(f: Node<'_>, ctx: &NativeContext<'_>) -> bool {
    let mut p = f.parent();
    while let Some(x) = p.filter(|x| x.kind() == "parenthesized_expression") {
        p = x.parent();
    }
    let Some(p) = p else { return false };
    match p.kind() {
        "call" => true,
        "keyword_argument" => p.child_by_field_name("name").is_some_and(|k| text(k, ctx.source) == "key"),
        "argument_list" => {
            let Some(call) = p.parent() else { return false };
            let Some(func) = call.child_by_field_name("function") else { return false };
            let simple = text(func, ctx.source);
            matches!(simple, "map" | "filter" | "reduce" | "functools.reduce")
                || model(ctx).is_some_and(|m| names_one_of(func, m, ctx.source, &["builtins.map", "builtins.filter", "functools.reduce"]))
        }
        _ => false,
    }
}

fn function_uses_loop_variable(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = model(ctx) else { return };
    let src = ctx.source;
    let (scope_part, body): (Node, Node) = match n.kind() {
        "for_statement" | "while_statement" => match n.child_by_field_name("body") {
            Some(b) => (n, b),
            None => return,
        },
        _ => (n, n),
    };
    let mut bound = BTreeSet::new();
    loop_bound_names(scope_part, src, &mut bound);
    if n.kind() == "while_statement" {
        bound.clear();
        loop_bound_names(body, src, &mut bound);
    }
    if bound.is_empty() {
        return;
    }
    let mut funcs = Vec::new();
    nested_functions(body, &mut funcs);
    for f in funcs {
        if used_immediately(f, ctx) {
            continue;
        }
        let fscope = m.scope_at(f.child_by_field_name("body").map(|b| b.start_byte()).unwrap_or(f.start_byte()));
        let mut reported = BTreeSet::new();
        for r in &m.references {
            if !(f.start_byte() <= r.start && r.start < f.end_byte() && r.read && bound.contains(&r.name)) {
                continue;
            }
            // Resolved outside the function (parameters and locals bind inside).
            let inside = r.symbol.is_some_and(|s| {
                let mut sc = Some(m.symbol(s).scope);
                while let Some(x) = sc {
                    if x == fscope {
                        return true;
                    }
                    sc = m.scopes[x].parent;
                }
                false
            });
            let in_default = f
                .child_by_field_name("parameters")
                .is_some_and(|p| p.start_byte() <= r.start && r.start < p.end_byte());
            if inside || in_default || m.scopes[m.symbol_scope(r)].kind == ScopeKind::Class {
                continue;
            }
            reported.insert(r.name.clone());
            out.push(NativeFinding {
                start: r.start,
                end: r.end,
                message: format!("This function reads the loop variable `{}` when it runs, not when it was defined", r.name),
                edits: Vec::new(),
            });
        }
    }
}
py_rule!(FunctionUsesLoopVariable, "py/function-uses-loop-variable", ["for_statement", "while_statement", "list_comprehension", "set_comprehension", "dictionary_comprehension", "generator_expression"], true, function_uses_loop_variable);

trait SymbolScope {
    fn symbol_scope(&self, r: &codelaw_semantic::Reference) -> usize;
}
impl SymbolScope for SemanticModel {
    fn symbol_scope(&self, r: &codelaw_semantic::Reference) -> usize {
        r.symbol.map(|s| self.symbol(s).scope).unwrap_or(0)
    }
}

const ABSTRACT: &[&str] = &["abc.abstractmethod", "abc.abstractproperty", "abc.abstractclassmethod", "abc.abstractstaticmethod"];

fn is_abstract_base(n: Node<'_>, ctx: &NativeContext<'_>) -> bool {
    let Some(m) = model(ctx) else { return false };
    let Some(sup) = n.child_by_field_name("superclasses") else { return false };
    let items = kids(sup);
    if items.len() != 1 {
        return false;
    }
    let it = items[0];
    if it.kind() == "keyword_argument" {
        return it.child_by_field_name("name").is_some_and(|k| text(k, ctx.source) == "metaclass")
            && it.child_by_field_name("value").is_some_and(|v| names_one_of(v, m, ctx.source, &["abc.ABCMeta"]) || local_named(v, m, ctx.source, "ABCMeta"));
    }
    names_one_of(it, m, ctx.source, &["abc.ABC"]) || local_named(it, m, ctx.source, "ABC")
}

/// A name defined in this file with the given text (inside the `abc` module itself).
fn local_named(n: Node<'_>, m: &SemanticModel, src: &str, name: &str) -> bool {
    n.kind() == "identifier"
        && text(n, src) == name
        && m.reference_at(n.start_byte()).and_then(|r| r.symbol).is_some_and(|s| m.symbol(s).kind != codelaw_semantic::SymbolKind::Import)
}

fn methods<'t>(class: Node<'t>) -> Vec<Node<'t>> {
    let Some(body) = class.child_by_field_name("body") else { return Vec::new() };
    statements(body)
        .into_iter()
        .filter_map(|s| match s.kind() {
            "function_definition" => Some(s),
            "decorated_definition" => s.child_by_field_name("definition").filter(|d| d.kind() == "function_definition"),
            _ => None,
        })
        .collect()
}

fn is_abstract_method(def: Node<'_>, ctx: &NativeContext<'_>) -> bool {
    let Some(m) = model(ctx) else { return false };
    decorators(def).into_iter().any(|d| {
        let t = decorator_target(d);
        names_one_of(t, m, ctx.source, ABSTRACT) || text(t, ctx.source).rsplit('.').next().is_some_and(|x| x.starts_with("abstract"))
    })
}

fn abstract_without_abstract_method(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if !is_abstract_base(n, ctx) {
        return;
    }
    let has_abstract = methods(n).into_iter().any(|d| is_abstract_method(d, ctx));
    let has_annotation = n.child_by_field_name("body").is_some_and(|b| {
        statements(b).iter().any(|s| {
            s.kind() == "expression_statement"
                && s.named_child(0).is_some_and(|a| a.kind() == "assignment" && a.child_by_field_name("right").is_none())
        })
    });
    if !has_abstract && !has_annotation
        && let Some(name) = n.child_by_field_name("name")
    {
        let t = text(name, ctx.source);
        out.push(hit(name, format!("`{t}` is an abstract base class with no abstract methods; mark what subclasses must define, or drop `ABC`")));
    }
}
py_rule!(AbstractBaseClassWithoutAbstractMethod, "py/abstract-base-class-without-abstract-method", ["class_definition"], true, abstract_without_abstract_method);

fn empty_method_without_abstract_decorator(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if !is_abstract_base(n, ctx) {
        return;
    }
    let Some(m) = model(ctx) else { return };
    let class = n.child_by_field_name("name").map(|x| text(x, ctx.source)).unwrap_or("");
    for d in methods(n) {
        if is_abstract_method(d, ctx) {
            continue;
        }
        let overload = decorators(d).into_iter().any(|x| names_one_of(decorator_target(x), m, ctx.source, &["typing.overload", "typing_extensions.overload"]));
        if overload || d.child_by_field_name("body").is_none_or(|b| !is_empty_body(b)) {
            continue;
        }
        if let Some(name) = d.child_by_field_name("name") {
            let t = text(name, ctx.source);
            out.push(hit(name, format!("`{class}.{t}` is empty in an abstract base class but not marked `@abstractmethod`")));
        }
    }
}
py_rule!(EmptyMethodWithoutAbstractDecorator, "py/empty-method-without-abstract-decorator", ["class_definition"], true, empty_method_without_abstract_decorator);

fn handler_names<'t>(h: Node<'t>) -> Vec<Node<'t>> {
    match handler_type(h) {
        Some(t) if t.kind() == "tuple" => kids(t).into_iter().filter(|c| c.kind() != "list_splat").collect(),
        Some(t) => vec![t],
        None => Vec::new(),
    }
}

fn duplicate_try_block_exception(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for h in kids(n).into_iter().filter(|c| c.kind() == "except_clause") {
        let names = handler_names(h);
        let mut here = BTreeSet::new();
        for x in names {
            let k = norm(x, ctx.source);
            if seen.contains(&k) {
                out.push(hit(x, format!("`{}` is already caught by an earlier handler of this `try`", text(x, ctx.source))));
            }
            here.insert(k);
        }
        seen.extend(here);
    }
}
py_rule!(DuplicateTryBlockException, "py/duplicate-try-block-exception", ["try_statement"], false, duplicate_try_block_exception);

fn star_arg_after_keyword(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let mut seen_kw = false;
    for c in kids(n) {
        match c.kind() {
            "keyword_argument" => seen_kw = true,
            "list_splat" if seen_kw => out.push(hit(c, "`*args` after a keyword argument fills the positions before it, which is confusing")),
            _ => {}
        }
    }
}
py_rule!(StarArgUnpackingAfterKeywordArg, "py/star-arg-unpacking-after-keyword-arg", ["argument_list"], false, star_arg_after_keyword);

fn no_explicit_stacklevel(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = model(ctx) else { return };
    let Some(f) = n.child_by_field_name("function") else { return };
    if !names_one_of(f, m, ctx.source, &["warnings.warn"]) {
        return;
    }
    let a = call_args(n, ctx.source);
    if a.starred || a.positional.len() >= 3 || a.keyword("stacklevel").is_some() || a.keyword("skip_file_prefixes").is_some() {
        return;
    }
    let Some(list) = a.list else { return };
    let close = children(list).into_iter().rev().find(|c| c.kind() == ")");
    let last = kids(list).last().copied();
    let edits = match (close, last) {
        (Some(_), Some(l)) => vec![TextEdit::new(l.end_byte(), l.end_byte(), ", stacklevel=2")],
        _ => Vec::new(),
    };
    out.push(hit_fix(n, "Pass `stacklevel=` so the warning points at the caller, not at this line", edits));
}
py_rule!(NoExplicitStacklevel, "py/no-explicit-stacklevel", ["call"], true, no_explicit_stacklevel);

fn except_with_empty_tuple(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if let Some(t) = handler_type(n)
        && t.kind() == "tuple"
        && kids(t).is_empty()
    {
        out.push(hit(t, "`except ():` catches nothing"));
    }
}
py_rule!(ExceptWithEmptyTuple, "py/except-with-empty-tuple", ["except_clause"], false, except_with_empty_tuple);

fn except_with_non_exception_classes(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(t) = handler_type(n) else { return };
    let mut items = Vec::new();
    flatten_handler(t, &mut items);
    for x in items {
        if !matches!(x.kind(), "identifier" | "attribute" | "call" | "subscript" | "list_splat" | "conditional_expression" | "boolean_operator") {
            out.push(hit(x, "`except` takes exception classes or tuples of them"));
        }
    }
}

fn flatten_handler<'t>(n: Node<'t>, out: &mut Vec<Node<'t>>) {
    let n = unparen(n);
    match n.kind() {
        "tuple" => {
            for c in kids(n) {
                flatten_handler(c, out);
            }
        }
        "list_splat" => {
            if let Some(inner) = n.named_child(0).filter(|i| matches!(i.kind(), "tuple" | "list")) {
                for c in kids(inner) {
                    flatten_handler(c, out);
                }
            } else {
                out.push(n);
            }
        }
        _ => out.push(n),
    }
}
py_rule!(ExceptWithNonExceptionClasses, "py/except-with-non-exception-classes", ["except_clause"], false, except_with_non_exception_classes);

fn in_nested_loop(r: usize, body: Node<'_>) -> bool {
    let Some(mut cur) = body.descendant_for_byte_range(r, r) else { return false };
    // The iterable of a `for` (and of a comprehension's first `for`) is
    // evaluated once, before the loop runs.
    let mut evaluated_once = false;
    while let Some(p) = cur.parent() {
        if p.id() == body.id() {
            return false;
        }
        match p.kind() {
            "for_statement" => {
                if p.child_by_field_name("right") != Some(cur) {
                    return true;
                }
            }
            "for_in_clause" => {
                let first = p.parent().and_then(|c| kids(c).into_iter().find(|k| k.kind() == "for_in_clause"));
                evaluated_once = p.child_by_field_name("right") == Some(cur) && first.is_some_and(|f| f.id() == p.id());
            }
            "while_statement" => return true,
            "list_comprehension" | "set_comprehension" | "dictionary_comprehension" | "generator_expression" => {
                if !evaluated_once {
                    return true;
                }
                evaluated_once = false;
            }
            _ => {}
        }
        cur = p;
    }
    false
}

fn reuse_of_groupby_generator(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = model(ctx) else { return };
    let (Some(left), Some(right), Some(body)) = (n.child_by_field_name("left"), n.child_by_field_name("right"), n.child_by_field_name("body")) else {
        return;
    };
    if right.kind() != "call" || !right.child_by_field_name("function").is_some_and(|f| names_one_of(f, m, ctx.source, &["itertools.groupby"])) {
        return;
    }
    let names: Vec<Node> = kids(unparen(left));
    if names.len() != 2 || names[1].kind() != "identifier" {
        return;
    }
    let group = names[1];
    let Some(sym) = m.declared_at(group.start_byte()) else { return };
    let mut reads: Vec<usize> = m.symbol(sym).refs.iter().map(|&r| &m.references[r]).filter(|r| r.read && body.start_byte() <= r.start && r.start < body.end_byte()).map(|r| r.start).collect();
    reads.sort();
    let mut counted: Vec<usize> = Vec::new();
    for r in reads {
        let node = body.descendant_for_byte_range(r, r);
        let again = counted.iter().any(|&c| match (node, body.descendant_for_byte_range(c, c)) {
            (Some(a), Some(b)) => !different_forks(b, a),
            _ => true,
        });
        if (in_nested_loop(r, body) || again)
            && let Some(nd) = node
        {
            out.push(hit(nd, format!("The group `{}` from `itertools.groupby()` is an iterator; using it again yields nothing", text(group, ctx.source))));
        }
        counted.push(r);
    }
}
py_rule!(ReuseOfGroupbyGenerator, "py/reuse-of-groupby-generator", ["for_statement"], true, reuse_of_groupby_generator);

fn unintentional_type_annotation(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(a) = n.named_child(0).filter(|a| a.kind() == "assignment") else { return };
    if a.child_by_field_name("right").is_some() || a.child_by_field_name("type").is_none() {
        return;
    }
    let suspicious = a.child_by_field_name("left").is_some_and(|l| {
        let base = match l.kind() {
            "attribute" => l.child_by_field_name("object"),
            "subscript" => l.child_by_field_name("value"),
            _ => None,
        };
        base.is_some_and(|b| b.kind() == "identifier" && (l.kind() == "subscript" || text(b, ctx.source) != "self"))
    });
    if suspicious {
        out.push(hit(a, "This annotates an attribute or item without assigning it; did you mean `=`?"));
    }
}
py_rule!(UnintentionalTypeAnnotation, "py/unintentional-type-annotation", ["expression_statement"], false, unintentional_type_annotation);

fn set_literal_key(n: Node<'_>, src: &str) -> Option<String> {
    let n = unparen(n);
    match n.kind() {
        "string" | "concatenated_string" if !is_fstring(n, src) => Some(format!("s{}", text(n, src))),
        "integer" | "float" | "true" | "false" | "none" => Some(format!("n{}", text(n, src))),
        _ => None,
    }
}

fn duplicate_value(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    for it in kids(n) {
        let Some(k) = set_literal_key(it, src) else { continue };
        if seen.insert(k, ()).is_some() {
            let edits = match (it.prev_sibling().filter(|c| c.kind() == ","), it.next_sibling().filter(|c| c.kind() == ",")) {
                (Some(prev), _) => vec![TextEdit::new(prev.start_byte(), it.end_byte(), "")],
                (None, Some(next)) => vec![TextEdit::new(it.start_byte(), next.end_byte(), "")],
                _ => Vec::new(),
            };
            out.push(hit_fix(it, format!("`{}` is already in this set", text(it, src)), edits));
        }
    }
}
py_rule!(DuplicateValue, "py/duplicate-set-value", ["set"], false, duplicate_value);

fn re_sub_positional_args(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(m) = model(ctx) else { return };
    let Some(f) = n.child_by_field_name("function") else { return };
    let Some(q) = qualified_name(f, m, ctx.source) else { return };
    let (max, params) = match q.as_str() {
        "re.sub" | "re.subn" => (3, "`count` and `flags`"),
        "re.split" => (2, "`maxsplit` and `flags`"),
        _ => return,
    };
    let a = call_args(n, ctx.source);
    if a.positional.len() > max && !a.starred {
        out.push(hit(n, format!("Pass {params} to `{q}` by keyword; their positions are easy to mix up")));
    }
}
py_rule!(ReSubPositionalArgs, "py/re-sub-positional-args", ["call"], true, re_sub_positional_args);

fn static_key_dict_comprehension(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    let Some(pair) = n.child_by_field_name("body").filter(|b| b.kind() == "pair") else { return };
    let Some(key) = pair.child_by_field_name("key") else { return };
    let mut bound = BTreeSet::new();
    for c in kids(n) {
        if c.kind() == "for_in_clause"
            && let Some(l) = c.child_by_field_name("left")
        {
            let mut names = Vec::new();
            target_names(l, &mut names);
            bound.extend(names.iter().map(|x| text(*x, src).to_owned()));
        }
    }
    let k = unparen(key);
    let static_key = match k.kind() {
        "string" | "concatenated_string" => !is_fstring(k, src) || {
            let mut names = Vec::new();
            loaded_names(k, &mut names);
            names.iter().all(|x| !bound.contains(text(*x, src)))
        },
        "integer" | "float" | "true" | "false" | "none" => true,
        "identifier" | "attribute" if attribute_chain(k) => {
            let mut names = Vec::new();
            loaded_names(k, &mut names);
            names.iter().all(|x| !bound.contains(text(*x, src)))
        }
        _ => false,
    };
    if static_key {
        out.push(hit(key, format!("Every item gets the same key `{}`, so the dict ends up with one entry", text(key, src))));
    }
}
py_rule!(StaticKeyDictComprehension, "py/static-key-dict-comprehension", ["dictionary_comprehension"], false, static_key_dict_comprehension);

fn raise_without_from_inside_except(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    let Some(exc) = kids(n).first().copied() else { return };
    if n.child_by_field_name("cause").is_some() {
        return;
    }
    let mut cur = n;
    let mut handler = None;
    while let Some(p) = cur.parent() {
        match p.kind() {
            "function_definition" | "lambda" | "class_definition" => return,
            "except_clause" => {
                handler = Some(p);
                break;
            }
            "finally_clause" | "else_clause" if p.parent().is_some_and(|g| g.kind() == "try_statement") => return,
            // Raised inside a nested `try`: that `try`'s own handlers deal with it.
            "try_statement" if p.child_by_field_name("body") == Some(cur) => return,
            _ => {}
        }
        cur = p;
    }
    let Some(h) = handler else { return };
    let bound = kids(h)
        .first()
        .filter(|f| f.kind() == "as_pattern")
        .and_then(|f| f.child_by_field_name("alias"))
        .map(|a| text(a, src).trim().to_owned());
    if exc.kind() == "identifier" && bound.as_deref() == Some(text(exc, src)) {
        return;
    }
    out.push(hit(n, "Inside `except`, write `raise … from err` (or `from None`) so the original error is kept or deliberately hidden"));
}
py_rule!(RaiseWithoutFromInsideExcept, "py/raise-without-from-inside-except", ["raise_statement"], false, raise_without_from_inside_except);

/// Whether the rule's `target_version` option (the oldest Python the code
/// supports) is at least `min`. Unset means yes.
fn targets_at_least(ctx: &NativeContext<'_>, min: (u32, u32)) -> bool {
    let Some(v) = ctx.options.get("target_version").and_then(|v| v.as_str()) else { return true };
    let v = v.trim_start_matches("py");
    let mut parts = v.split('.');
    let major: u32 = parts.next().and_then(|x| x.parse().ok()).unwrap_or(3);
    let minor: u32 = parts.next().and_then(|x| x.parse().ok()).unwrap_or(0);
    (major, minor) >= min
}

fn add_strict(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>, name: &str, min_args: usize, min: (u32, u32)) {
    let Some(m) = model(ctx) else { return };
    let Some(f) = n.child_by_field_name("function") else { return };
    if !targets_at_least(ctx, min) || !names_one_of(f, m, ctx.source, &[name]) {
        return;
    }
    let a = call_args(n, ctx.source);
    let splats = a.list.map(|l| kids(l).iter().filter(|c| c.kind() == "list_splat").count()).unwrap_or(0);
    let has_kw_splat = a.list.is_some_and(|l| kids(l).iter().any(|c| c.kind() == "dictionary_splat"));
    if has_kw_splat || a.positional.len() + splats < min_args || a.keyword("strict").is_some() {
        return;
    }
    let infinite = a.positional.iter().any(|p| {
        p.kind() == "call"
            && p.child_by_field_name("function").is_some_and(|pf| {
                names_one_of(pf, m, ctx.source, &["itertools.count", "itertools.cycle"])
                    || (names_one_of(pf, m, ctx.source, &["itertools.repeat"]) && call_args(*p, ctx.source).positional.len() == 1)
            })
    });
    if infinite {
        return;
    }
    let Some(last) = a.list.and_then(|l| kids(l).last().copied()) else { return };
    let short = name.rsplit('.').next().unwrap_or(name);
    out.push(hit_fix(
        n,
        format!("Pass `strict=` to `{short}()`: `strict=True` to catch inputs of different lengths, `strict=False` to keep today's behaviour"),
        vec![TextEdit::new(last.end_byte(), last.end_byte(), ", strict=False")],
    ));
}

fn zip_without_strict(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    add_strict(n, ctx, out, "builtins.zip", 1, (3, 10))
}
fn batched_without_strict(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    add_strict(n, ctx, out, "itertools.batched", 1, (3, 13))
}
fn map_without_strict(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    add_strict(n, ctx, out, "builtins.map", 3, (3, 14))
}
py_rule!(ZipWithoutExplicitStrict, "py/zip-without-explicit-strict", ["call"], true, zip_without_strict);
py_rule!(BatchedWithoutExplicitStrict, "py/batched-without-explicit-strict", ["call"], true, batched_without_strict);
py_rule!(MapWithoutExplicitStrict, "py/map-without-explicit-strict", ["call"], true, map_without_strict);

#[cfg(test)]
mod tests {
    use crate::run;
    use codelaw_core::Language;

    fn spans(rule: &dyn crate::NativeRule, src: &str) -> Vec<String> {
        run(rule, Language::Python, src).iter().map(|f| src[f.start..f.end].to_owned()).collect()
    }

    #[test]
    fn loop_variable_closures() {
        let src = "for i in range(3):\n    fs = lambda: i\n    def inner():\n        return i\n    g(lambda: i)\n    sorted(z, key=lambda t: t[i])\n    h = lambda i=i: i\n";
        assert_eq!(spans(&super::FunctionUsesLoopVariable, src), ["i", "i", "i"]);
    }

    #[test]
    fn defaults() {
        let src = "import re\ndef f(a=[], b=set(), c=foo(), d=tuple(), e=re.compile('x'), f=lambda: bar()):\n    pass\n";
        assert_eq!(spans(&super::MutableArgumentDefault, src), ["[]", "set()"]);
        assert_eq!(spans(&super::FunctionCallInDefaultArgument, src), ["foo()"]);
    }

    #[test]
    fn abstract_classes() {
        let src = "from abc import ABC, abstractmethod\nclass A(ABC):\n    def m(self): pass\nclass B(ABC):\n    x: int\nclass C(ABC):\n    @abstractmethod\n    def m(self): ...\n    def e(self): pass\n";
        assert_eq!(spans(&super::AbstractBaseClassWithoutAbstractMethod, src), ["A"]);
        assert_eq!(spans(&super::EmptyMethodWithoutAbstractDecorator, src), ["m", "e"]);
    }
}
