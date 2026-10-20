//! Helpers the Python native rules share: string literals, operators,
//! qualified names resolved through imports, and scope boundaries.

use crate::util::{children, named_children, text};
use codelaw_core::TextEdit;
use codelaw_semantic::{Imported, SemanticModel, SymbolKind};
use tree_sitter::Node;

pub const PY: &[codelaw_core::Language] = &[codelaw_core::Language::Python];

/// Named children without comments.
pub fn kids<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    named_children(n).into_iter().filter(|c| c.kind() != "comment").collect()
}

/// Strip parentheses around an expression.
pub fn unparen(mut n: Node<'_>) -> Node<'_> {
    while n.kind() == "parenthesized_expression" {
        match kids(n).first() {
            Some(c) => n = *c,
            None => break,
        }
    }
    n
}

/// A string node's prefix (lower-cased) and its literal content, with
/// escapes left as written. `None` when it has interpolations.
pub fn string_parts(n: Node<'_>, src: &str) -> Option<(String, String)> {
    if n.kind() != "string" {
        return None;
    }
    let mut prefix = String::new();
    let mut content = String::new();
    for c in children(n) {
        match c.kind() {
            "string_start" => {
                prefix = text(c, src).trim_end_matches(['"', '\'']).to_ascii_lowercase();
            }
            "string_content" => content.push_str(text(c, src)),
            "escape_sequence" => content.push_str(text(c, src)),
            "interpolation" => return None,
            _ => {}
        }
    }
    Some((prefix, content))
}

/// A plain string literal's value, escapes decoded for the common cases;
/// implicit concatenation joined. `None` for f-strings and bytes.
pub fn string_value(n: Node<'_>, src: &str) -> Option<String> {
    match n.kind() {
        "string" => {
            let (prefix, content) = string_parts(n, src)?;
            if prefix.contains('f') || prefix.contains('b') || prefix.contains('t') {
                return None;
            }
            Some(if prefix.contains('r') { content } else { unescape(&content) })
        }
        "concatenated_string" => {
            let mut out = String::new();
            for c in kids(n) {
                out.push_str(&string_value(c, src)?);
            }
            Some(out)
        }
        _ => None,
    }
}

/// Decode the backslash escapes a lint needs to read a string's value.
pub fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('0') => out.push('\0'),
            Some('\\') => out.push('\\'),
            Some('\'') => out.push('\''),
            Some('"') => out.push('"'),
            Some('\n') => {}
            Some(o) => {
                out.push('\\');
                out.push(o);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Whether a node is a string literal of any kind.
pub fn is_string(n: Node<'_>) -> bool {
    matches!(n.kind(), "string" | "concatenated_string")
}

/// Whether a string node (or any part of a concatenation) is an f-string
/// or t-string.
pub fn is_fstring(n: Node<'_>, src: &str) -> bool {
    match n.kind() {
        "string" => children(n)
            .iter()
            .find(|c| c.kind() == "string_start")
            .is_some_and(|s| text(*s, src).chars().take_while(|c| c.is_ascii_alphabetic()).any(|c| matches!(c, 'f' | 'F' | 't' | 'T'))),
        "concatenated_string" => kids(n).iter().any(|c| is_fstring(*c, src)),
        _ => false,
    }
}

/// The name an expression resolves to through the file's imports:
/// `os.path.join` for `from os import path; path.join`, `builtins.len`
/// for the builtin `len`. `None` for local names and anything dynamic.
pub fn qualified_name(n: Node<'_>, model: &SemanticModel, src: &str) -> Option<String> {
    let n = unparen(n);
    match n.kind() {
        "identifier" => {
            let name = text(n, src);
            match model.reference_at(n.start_byte()) {
                Some(r) => match r.symbol {
                    None => {
                        if codelaw_semantic::PYTHON_BUILTINS.contains(&name) {
                            Some(format!("builtins.{name}"))
                        } else {
                            None
                        }
                    }
                    Some(id) => {
                        let sym = model.symbol(id);
                        if sym.kind != SymbolKind::Import {
                            return None;
                        }
                        let imp = model.import_of(id)?;
                        match &imp.imported {
                            Imported::Module => {
                                // `import a.b` binds `a`; `import a.b as c` binds `a.b`.
                                let aliased = src[imp.start..imp.end].contains(" as ");
                                Some(if aliased { imp.source.clone() } else { imp.source.split('.').next().unwrap_or("").to_owned() })
                            }
                            Imported::Named(x) => Some(format!("{}.{}", imp.source, x)),
                            _ => None,
                        }
                    }
                },
                None => None,
            }
        }
        "attribute" => {
            let obj = n.child_by_field_name("object")?;
            let attr = n.child_by_field_name("attribute")?;
            Some(format!("{}.{}", qualified_name(obj, model, src)?, text(attr, src)))
        }
        _ => None,
    }
}

/// Whether `n` names one of `names` (qualified, e.g. `functools.cache`).
pub fn names_one_of(n: Node<'_>, model: &SemanticModel, src: &str, names: &[&str]) -> bool {
    qualified_name(n, model, src).is_some_and(|q| names.contains(&q.as_str()))
}

/// A call's positional arguments, keyword arguments and whether it has
/// `*args` or `**kwargs`.
pub struct CallArgs<'t> {
    pub positional: Vec<Node<'t>>,
    pub keywords: Vec<(String, Node<'t>, Node<'t>)>,
    pub starred: bool,
    pub list: Option<Node<'t>>,
}

pub fn call_args<'t>(call: Node<'t>, src: &str) -> CallArgs<'t> {
    let mut a = CallArgs { positional: Vec::new(), keywords: Vec::new(), starred: false, list: None };
    let Some(list) = call.child_by_field_name("arguments") else { return a };
    if list.kind() != "argument_list" {
        // `f(x for x in y)`: one generator argument.
        a.positional.push(list);
        return a;
    }
    a.list = Some(list);
    for c in kids(list) {
        match c.kind() {
            "keyword_argument" => {
                if let (Some(k), Some(v)) = (c.child_by_field_name("name"), c.child_by_field_name("value")) {
                    a.keywords.push((text(k, src).to_owned(), v, c));
                }
            }
            "list_splat" | "dictionary_splat" => a.starred = true,
            _ => a.positional.push(c),
        }
    }
    a
}

impl<'t> CallArgs<'t> {
    pub fn keyword(&self, name: &str) -> Option<Node<'t>> {
        self.keywords.iter().find(|k| k.0 == name).map(|k| k.1)
    }
}

/// The enclosing function or class definition, or `None` at module level.
pub fn enclosing_def(n: Node<'_>) -> Option<Node<'_>> {
    let mut p = n.parent();
    while let Some(x) = p {
        if matches!(x.kind(), "function_definition" | "lambda" | "class_definition") {
            return Some(x);
        }
        p = x.parent();
    }
    None
}

/// The decorators of a function or class definition.
pub fn decorators<'t>(def: Node<'t>) -> Vec<Node<'t>> {
    match def.parent() {
        Some(p) if p.kind() == "decorated_definition" => {
            kids(p).into_iter().filter(|c| c.kind() == "decorator").filter_map(|d| kids(d).first().copied()).collect()
        }
        _ => Vec::new(),
    }
}

/// A decorator expression without its call: `@lru_cache(maxsize=1)` is
/// `lru_cache`.
pub fn decorator_target(d: Node<'_>) -> Node<'_> {
    if d.kind() == "call"
        && let Some(f) = d.child_by_field_name("function")
    {
        return f;
    }
    d
}

/// The statements of a block, comments left out.
pub fn statements<'t>(block: Node<'t>) -> Vec<Node<'t>> {
    kids(block)
}

/// Whether a statement is a docstring (a lone string expression).
pub fn is_docstring_stmt(n: Node<'_>) -> bool {
    n.kind() == "expression_statement" && n.named_child_count() == 1 && n.named_child(0).is_some_and(|c| is_string(c))
}

/// The indentation of the line `offset` is on.
pub fn indent_at(src: &str, offset: usize) -> String {
    let line_start = src[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    src[line_start..].chars().take_while(|c| *c == ' ' || *c == '\t').collect()
}

/// Whether a statement starts its own line (nothing but indentation before
/// it), so it can be rewritten line-wise.
pub fn starts_line(src: &str, offset: usize) -> bool {
    let line_start = src[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    src[line_start..offset].trim().is_empty()
}

/// Whether an expression's evaluation can have no visible effect: names,
/// literals, attribute chains and containers of them.
pub fn is_pure(n: Node<'_>) -> bool {
    match n.kind() {
        "identifier" | "integer" | "float" | "true" | "false" | "none" | "ellipsis" | "string" | "concatenated_string" => {
            n.kind() != "string" || kids(n).iter().all(|c| c.kind() != "interpolation")
        }
        "parenthesized_expression" | "tuple" | "list" | "set" | "expression_list" => kids(n).iter().all(|c| is_pure(*c)),
        "dictionary" => kids(n).iter().all(|p| p.kind() == "pair" && kids(*p).iter().all(|c| is_pure(*c))),
        "unary_operator" => n.child_by_field_name("argument").is_some_and(is_pure),
        _ => false,
    }
}

/// Implement `NativeRule` for a Python rule from a function over the node.
macro_rules! py_rule {
    ($ty:ident, $id:literal, [$($k:literal),* $(,)?], $sem:literal, $f:path) => {
        pub struct $ty;
        impl crate::NativeRule for $ty {
            fn id(&self) -> &'static str {
                $id
            }
            fn languages(&self) -> &'static [codelaw_core::Language] {
                crate::python_util::PY
            }
            fn kinds(&self) -> &'static [&'static str] {
                &[$($k),*]
            }
            fn needs_semantic(&self) -> bool {
                $sem
            }
            fn check(&self, node: codelaw_lang::SyntaxNode<'_>, ctx: &crate::NativeContext<'_>, out: &mut Vec<crate::NativeFinding>) {
                $f(node.get_inner_node(), ctx, out)
            }
        }
    };
}
pub(crate) use py_rule;

/// A finding over a node, without edits.
pub fn hit(n: Node<'_>, message: impl Into<String>) -> crate::NativeFinding {
    crate::NativeFinding { start: n.start_byte(), end: n.end_byte(), message: message.into(), edits: Vec::new() }
}

/// A finding over a node, with edits.
pub fn hit_fix(n: Node<'_>, message: impl Into<String>, edits: Vec<TextEdit>) -> crate::NativeFinding {
    crate::NativeFinding { start: n.start_byte(), end: n.end_byte(), message: message.into(), edits }
}

/// The edits that delete a statement: its whole line, or `pass` when it is
/// the only statement of its block. `None` when it shares its line.
pub fn remove_stmt(stmt: Node<'_>, src: &str) -> Option<Vec<TextEdit>> {
    if !starts_line(src, stmt.start_byte()) {
        return None;
    }
    let end = stmt.end_byte();
    let line_end = src[end..].find('\n').map(|i| end + i).unwrap_or(src.len());
    let rest = src[end..line_end].trim();
    if !rest.is_empty() && !rest.starts_with('#') {
        return None;
    }
    let parent = stmt.parent()?;
    if parent.kind() == "block" && statements(parent).len() == 1 {
        return Some(vec![TextEdit::new(stmt.start_byte(), stmt.end_byte(), "pass")]);
    }
    let line_start = src[..stmt.start_byte()].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let stop = if line_end < src.len() { line_end + 1 } else { line_end };
    Some(vec![TextEdit::new(line_start, stop, "")])
}

/// The statement a node belongs to (its nearest ancestor that is a child
/// of a block or the module).
pub fn statement_of(n: Node<'_>) -> Node<'_> {
    let mut cur = n;
    while let Some(p) = cur.parent() {
        if matches!(p.kind(), "block" | "module") {
            return cur;
        }
        cur = p;
    }
    cur
}

/// Whether a block's statements are all `pass`, `...` or a docstring.
pub fn is_empty_body(block: Node<'_>) -> bool {
    statements(block).iter().all(|s| {
        s.kind() == "pass_statement"
            || (s.kind() == "expression_statement" && s.named_child(0).is_some_and(|c| c.kind() == "ellipsis" || is_string(c)))
    })
}
