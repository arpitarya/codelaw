//! A file's flag facts (SR-PROJECT-GRAPH decision 11): its calls with
//! their arguments, its named functions and their parameters, its string
//! constants and its `codelaw-flag` markers. Read once per file and cached
//! by content, like the graph's facts.

use crate::facts::Span;
use codelaw_core::Language;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tree_sitter::Node;

/// The facts of one file that the flag rules read.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlagFacts {
    pub python: bool,
    pub calls: Vec<Call>,
    pub funcs: Vec<Func>,
    /// Top-level string constants: `X`, `Flags.NEW` (an enum member or an
    /// object literal's key) → the string.
    pub consts: Vec<(String, String)>,
    pub markers: Vec<Marker>,
}

/// One call.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Call {
    /// The callee as a path: `["client", "variation"]`; the first segment is
    /// `""` when the receiver is no name (`useClient().variation`).
    pub callee: Vec<String>,
    pub args: Vec<Arg>,
    pub span: Span,
    pub line: u32,
    /// The innermost function the call is in.
    pub func: Option<u32>,
    /// The edits that keep one branch, when the call is the bare condition
    /// of an `if` or a conditional expression and its arguments have no
    /// side effect.
    pub collapse: Option<Collapse>,
}

/// One argument, as far as a flag name can be read from it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Arg {
    /// A string literal with nothing interpolated.
    Str(String, Span),
    /// A name or a member path: `NEW_CHECKOUT`, `Flags.NEW`.
    Name(Vec<String>, Span),
    /// A parameter of an enclosing function (`func`, its `index`), never
    /// reassigned in it: the value flows through unchanged.
    Param { func: u32, index: u32, span: Span },
    /// A JS object literal: its keys and their values.
    Object(Vec<(String, Arg)>, Span),
    /// A Python keyword argument.
    Keyword(String, Box<Arg>, Span),
    /// `...xs`, `*xs`, `**kw`: positions after it are unknown.
    Spread(Span),
    Other(Span),
}

impl Arg {
    pub fn span(&self) -> Span {
        match self {
            Arg::Str(_, s) | Arg::Name(_, s) | Arg::Object(_, s) | Arg::Keyword(_, _, s) | Arg::Spread(s) | Arg::Other(s) => *s,
            Arg::Param { span, .. } => *span,
        }
    }
}

/// A function with parameters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Func {
    /// Empty for an anonymous function.
    pub name: String,
    /// A class method or an object literal's function property: called as
    /// `x.name(…)`.
    pub method: bool,
    /// Declared at the module's top level, so it can be imported.
    pub top: bool,
    /// The parameters as called (`self`/`cls` dropped); `""` for one that
    /// is no plain name, `"*"` for a rest parameter.
    pub params: Vec<String>,
    pub line: u32,
}

/// A `codelaw-flag <name> expires <date> owner <o> ticket <t> keep on|off
/// reason <text>` comment.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Marker {
    pub name: String,
    pub expires: Option<String>,
    pub owner: Option<String>,
    pub ticket: Option<String>,
    pub keep: Option<String>,
    pub reason: Option<String>,
    /// From `codelaw-flag` to the end of the marker's text.
    pub span: Span,
    /// The whole comment, with the line when it is alone on it.
    pub removal: Span,
    pub line: u32,
    /// Why the marker cannot be read.
    pub bad: Option<String>,
}

/// The edit that keeps one branch of a condition on a flag.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collapse {
    /// Keep the branch taken when the flag is on.
    pub on: (Span, String),
    /// Keep the branch taken when it is off.
    pub off: (Span, String),
}

/// The marker word.
pub const MARKER: &str = "codelaw-flag";

/// The flag facts of a JS, TS or Python file; `None` for other languages.
pub fn extract_flags(language: Language, text: &str) -> Option<FlagFacts> {
    let python = language == Language::Python;
    if !python && !language.is_js_family() {
        return None;
    }
    let root = codelaw_lang::parse(language, text);
    let node = root.root().get_inner_node();
    let mut x = Extract { text, python, f: FlagFacts { python, ..Default::default() }, stack: Vec::new() };
    x.consts(node);
    x.walk(node, 0);
    Some(x.f)
}

fn named<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    let mut c = n.walk();
    n.named_children(&mut c).filter(|c| c.kind() != "comment").collect()
}

fn line_of(text: &str, at: usize) -> u32 {
    text[..at.min(text.len())].matches('\n').count() as u32 + 1
}

fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map(|i| i + 1).unwrap_or(0)
}

/// Only whitespace between the start of `at`'s line and `at`.
fn alone_before(text: &str, at: usize) -> bool {
    text[line_start(text, at)..at].trim().is_empty()
}

/// A statement's span widened to its whole lines, newline included, when
/// nothing else is on them.
fn whole_lines(text: &str, s: usize, e: usize) -> Span {
    let ls = line_start(text, s);
    let start = if text[ls..s].trim().is_empty() { ls } else { s };
    let le = text[e..].find('\n').map(|i| e + i + 1).unwrap_or(text.len());
    let end = if text[e..le].trim().is_empty() { le } else { e };
    (start, end)
}

/// Lines re-indented so the first starts at the column of the statement it
/// replaces and the others keep their shape under `indent`.
fn reindent(lines: &[&str], indent: &str) -> String {
    let lines: Vec<&str> = lines.iter().map(|l| l.trim_end()).collect();
    let min = lines.iter().filter(|l| !l.trim().is_empty()).map(|l| l.len() - l.trim_start().len()).min().unwrap_or(0);
    let mut out = String::new();
    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
            if !l.trim().is_empty() {
                out.push_str(indent);
            }
        }
        if !l.trim().is_empty() {
            out.push_str(&l[min.min(l.len())..]);
        }
    }
    out
}

/// The lines of a block's inside, blank ones at either end dropped.
fn inner_lines(s: &str) -> Vec<&str> {
    let mut v: Vec<&str> = s.split('\n').collect();
    while v.first().is_some_and(|l| l.trim().is_empty()) {
        v.remove(0);
    }
    while v.last().is_some_and(|l| l.trim().is_empty()) {
        v.pop();
    }
    v
}

struct Frame {
    func: u32,
    params: Vec<String>,
    assigned: BTreeSet<String>,
}

struct Extract<'a> {
    text: &'a str,
    python: bool,
    f: FlagFacts,
    stack: Vec<Frame>,
}

const MAX_DEPTH: usize = 400;

impl<'a> Extract<'a> {
    fn t(&self, n: Node<'_>) -> &'a str {
        &self.text[n.byte_range()]
    }

    fn span(n: Node<'_>) -> Span {
        (n.start_byte(), n.end_byte())
    }

    fn walk(&mut self, n: Node<'_>, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        if n.kind() == "comment" {
            self.marker(n);
            return;
        }
        let pushed = self.enter_function(n);
        if matches!(n.kind(), "call_expression" | "call") {
            self.call(n);
        }
        let mut c = n.walk();
        let kids: Vec<Node<'_>> = n.children(&mut c).collect();
        for k in kids {
            self.walk(k, depth + 1);
        }
        if pushed {
            self.stack.pop();
        }
    }

    // ---- functions -------------------------------------------------------

    fn enter_function(&mut self, n: Node<'_>) -> bool {
        let is_fn = if self.python {
            matches!(n.kind(), "function_definition" | "lambda")
        } else {
            matches!(
                n.kind(),
                "function_declaration" | "generator_function_declaration" | "function_expression" | "function" | "generator_function" | "arrow_function" | "method_definition"
            )
        };
        if !is_fn {
            return false;
        }
        let (name, method, top) = self.function_name(n);
        let mut params = self.params(n);
        if self.python && method && params.first().is_some_and(|p| p == "self" || p == "cls") {
            params.remove(0);
        }
        let mut assigned = BTreeSet::new();
        if let Some(body) = n.child_by_field_name("body") {
            self.assignments(body, &mut assigned, 0);
        }
        let func = self.f.funcs.len() as u32;
        self.f.funcs.push(Func { name, method, top, params: params.clone(), line: line_of(self.text, n.start_byte()) });
        self.stack.push(Frame { func, params, assigned });
        true
    }

    /// The function's name, whether it is called as a method, and whether it
    /// is declared at the top level.
    fn function_name(&self, n: Node<'_>) -> (String, bool, bool) {
        let top_level = |decl: Node<'_>| -> bool {
            let mut p = decl.parent();
            if p.is_some_and(|p| matches!(p.kind(), "export_statement" | "decorated_definition")) {
                p = p.and_then(|p| p.parent());
            }
            p.is_some_and(|p| matches!(p.kind(), "program" | "module"))
        };
        if self.python {
            if n.kind() == "lambda" {
                return (String::new(), false, false);
            }
            let name = n.child_by_field_name("name").map(|x| self.t(x).to_owned()).unwrap_or_default();
            let mut owner = n.parent();
            if owner.is_some_and(|p| p.kind() == "decorated_definition") {
                owner = owner.and_then(|p| p.parent());
            }
            let method = owner.is_some_and(|b| b.kind() == "block" && b.parent().is_some_and(|c| c.kind() == "class_definition"));
            return (name, method, top_level(n));
        }
        match n.kind() {
            "function_declaration" | "generator_function_declaration" => {
                (n.child_by_field_name("name").map(|x| self.t(x).to_owned()).unwrap_or_default(), false, top_level(n))
            }
            "method_definition" => (n.child_by_field_name("name").map(|x| self.t(x).to_owned()).unwrap_or_default(), true, false),
            _ => {
                let Some(p) = n.parent() else { return (String::new(), false, false) };
                match p.kind() {
                    "variable_declarator" => {
                        let name = p.child_by_field_name("name").filter(|x| x.kind() == "identifier").map(|x| self.t(x).to_owned()).unwrap_or_default();
                        let top = p.parent().is_some_and(top_level);
                        (name, false, top)
                    }
                    "pair" => {
                        let key = p.child_by_field_name("key").map(|k| self.t(k).trim_matches(['"', '\'']).to_owned()).unwrap_or_default();
                        (key, true, false)
                    }
                    "public_field_definition" | "field_definition" => {
                        let key = p.child_by_field_name("name").or_else(|| p.child_by_field_name("property")).map(|k| self.t(k).to_owned()).unwrap_or_default();
                        (key, true, false)
                    }
                    _ => (String::new(), false, false),
                }
            }
        }
    }

    fn params(&self, n: Node<'_>) -> Vec<String> {
        let list = n.child_by_field_name("parameters");
        let Some(list) = list else {
            // `x => …`
            return n.child_by_field_name("parameter").map(|p| vec![self.t(p).to_owned()]).unwrap_or_default();
        };
        let mut out = Vec::new();
        for p in named(list) {
            let name = match p.kind() {
                "identifier" => self.t(p).to_owned(),
                "required_parameter" | "optional_parameter" => {
                    p.child_by_field_name("pattern").filter(|x| x.kind() == "identifier").map(|x| self.t(x).to_owned()).unwrap_or_default()
                }
                "assignment_pattern" => p.child_by_field_name("left").filter(|x| x.kind() == "identifier").map(|x| self.t(x).to_owned()).unwrap_or_default(),
                "typed_parameter" => named(p).into_iter().find(|x| x.kind() == "identifier").map(|x| self.t(x).to_owned()).unwrap_or_default(),
                "default_parameter" | "typed_default_parameter" => {
                    p.child_by_field_name("name").filter(|x| x.kind() == "identifier").map(|x| self.t(x).to_owned()).unwrap_or_default()
                }
                "rest_pattern" | "list_splat_pattern" | "dictionary_splat_pattern" => "*".to_owned(),
                "keyword_separator" | "positional_separator" => continue,
                _ => String::new(),
            };
            out.push(name);
        }
        out
    }

    /// Every name assigned in a function body: a parameter assigned there
    /// no longer flows unchanged.
    fn assignments(&self, n: Node<'_>, out: &mut BTreeSet<String>, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        let left = match n.kind() {
            "assignment_expression" | "augmented_assignment_expression" | "assignment" | "augmented_assignment" => n.child_by_field_name("left"),
            "update_expression" => n.child_by_field_name("argument"),
            "for_statement" | "for_in_statement" if self.python || n.kind() == "for_in_statement" => n.child_by_field_name("left"),
            _ => None,
        };
        if let Some(l) = left {
            self.names_in(l, out, 0);
        }
        let mut c = n.walk();
        let kids: Vec<Node<'_>> = n.children(&mut c).collect();
        for k in kids {
            self.assignments(k, out, depth + 1);
        }
    }

    fn names_in(&self, n: Node<'_>, out: &mut BTreeSet<String>, depth: usize) {
        if depth > 20 {
            return;
        }
        if matches!(n.kind(), "identifier" | "shorthand_property_identifier_pattern") {
            out.insert(self.t(n).to_owned());
            return;
        }
        if matches!(n.kind(), "member_expression" | "attribute" | "subscript_expression" | "subscript") {
            return;
        }
        for k in named(n) {
            self.names_in(k, out, depth + 1);
        }
    }

    // ---- calls -----------------------------------------------------------

    fn callee(&self, n: Node<'_>) -> Vec<String> {
        let (member, object, prop) = if self.python { ("attribute", "object", "attribute") } else { ("member_expression", "object", "property") };
        let mut segs = Vec::new();
        let mut cur = n;
        for _ in 0..32 {
            if cur.kind() == member {
                let Some(p) = cur.child_by_field_name(prop) else { return Vec::new() };
                segs.push(self.t(p).to_owned());
                let Some(o) = cur.child_by_field_name(object) else { return Vec::new() };
                cur = o;
                continue;
            }
            match cur.kind() {
                "identifier" | "this" => segs.push(self.t(cur).to_owned()),
                "non_null_expression" | "parenthesized_expression" => {
                    if let Some(i) = named(cur).into_iter().next() {
                        cur = i;
                        continue;
                    }
                    segs.push(String::new());
                }
                _ => segs.push(String::new()),
            }
            break;
        }
        segs.reverse();
        segs
    }

    fn call(&mut self, n: Node<'_>) {
        let Some(function) = n.child_by_field_name("function") else { return };
        let callee = self.callee(function);
        if callee.is_empty() {
            return;
        }
        let Some(list) = n.child_by_field_name("arguments") else { return };
        if !matches!(list.kind(), "arguments" | "argument_list") {
            return;
        }
        let args: Vec<Arg> = named(list).into_iter().map(|a| self.arg(a)).collect();
        let pure = named(list).into_iter().all(|a| self.pure(a, 0));
        let collapse = if pure { self.collapse(n) } else { None };
        let func = self.stack.last().map(|f| f.func);
        self.f.calls.push(Call { callee, args, span: Self::span(n), line: line_of(self.text, n.start_byte()), func, collapse });
    }

    fn string(&self, n: Node<'_>) -> Option<String> {
        if self.python {
            if n.kind() != "string" {
                return None;
            }
            let kids = named(n);
            let start = kids.first().filter(|k| k.kind() == "string_start").map(|k| self.t(*k)).unwrap_or("\"");
            if start.contains(['f', 'F', 'b', 'B']) || kids.iter().any(|k| k.kind() == "interpolation") {
                return None;
            }
            return Some(kids.iter().filter(|k| k.kind() == "string_content").map(|k| self.t(*k)).collect());
        }
        match n.kind() {
            "string" => Some(named(n).iter().filter(|k| k.kind() == "string_fragment").map(|k| self.t(*k)).collect()),
            "template_string" if !named(n).iter().any(|k| k.kind() == "template_substitution") => {
                Some(named(n).iter().filter(|k| k.kind() == "string_fragment").map(|k| self.t(*k)).collect())
            }
            _ => None,
        }
    }

    fn path(&self, n: Node<'_>) -> Option<Vec<String>> {
        let p = self.callee(n);
        (!p.is_empty() && p.iter().all(|s| !s.is_empty() && s != "this")).then_some(p)
    }

    fn arg(&self, n: Node<'_>) -> Arg {
        let span = Self::span(n);
        if let Some(s) = self.string(n) {
            return Arg::Str(s, span);
        }
        match n.kind() {
            "identifier" | "shorthand_property_identifier" => {
                let name = self.t(n);
                for f in self.stack.iter().rev() {
                    if let Some(i) = f.params.iter().position(|p| p == name) {
                        if f.assigned.contains(name) {
                            return Arg::Other(span);
                        }
                        return Arg::Param { func: f.func, index: i as u32, span };
                    }
                }
                Arg::Name(vec![name.to_owned()], span)
            }
            "member_expression" | "attribute" => match self.path(n) {
                Some(p) => Arg::Name(p, span),
                None => Arg::Other(span),
            },
            "parenthesized_expression" | "as_expression" | "satisfies_expression" | "non_null_expression" => {
                named(n).into_iter().next().map(|i| self.arg(i)).unwrap_or(Arg::Other(span))
            }
            "object" => {
                let mut keys = Vec::new();
                for p in named(n) {
                    match p.kind() {
                        "pair" => {
                            let (Some(k), Some(v)) = (p.child_by_field_name("key"), p.child_by_field_name("value")) else { continue };
                            let key = self.string(k).unwrap_or_else(|| self.t(k).to_owned());
                            keys.push((key, self.arg(v)));
                        }
                        "shorthand_property_identifier" => keys.push((self.t(p).to_owned(), self.arg(p))),
                        _ => keys.push((String::new(), Arg::Spread(Self::span(p)))),
                    }
                }
                Arg::Object(keys, span)
            }
            "keyword_argument" => {
                let name = n.child_by_field_name("name").map(|x| self.t(x).to_owned()).unwrap_or_default();
                let value = n.child_by_field_name("value").map(|v| self.arg(v)).unwrap_or(Arg::Other(span));
                Arg::Keyword(name, Box::new(value), span)
            }
            "spread_element" | "list_splat" | "dictionary_splat" => Arg::Spread(span),
            _ => Arg::Other(span),
        }
    }

    /// An argument with no side effect: a literal, a name, a member path, or
    /// an object, array or keyword argument of those.
    fn pure(&self, n: Node<'_>, depth: usize) -> bool {
        if depth > 8 {
            return false;
        }
        match n.kind() {
            "string" | "template_string" => self.string(n).is_some(),
            "identifier" | "shorthand_property_identifier" | "number" | "integer" | "float" | "true" | "false" | "null" | "undefined" | "none" | "this" => true,
            "member_expression" | "attribute" => self.path(n).is_some(),
            "object" | "array" | "list" | "tuple" | "dictionary" | "pair" | "keyword_argument" | "parenthesized_expression" | "as_expression" => {
                named(n).into_iter().all(|k| matches!(k.kind(), "property_identifier") || self.pure(k, depth + 1))
            }
            _ => false,
        }
    }

    /// The collapse edits, when the call is the whole condition.
    fn collapse(&self, call: Node<'_>) -> Option<Collapse> {
        let mut cond = call;
        let mut parent = call.parent()?;
        if !self.python && parent.kind() == "parenthesized_expression" {
            cond = parent;
            parent = parent.parent()?;
        }
        let is_condition = parent.child_by_field_name("condition") == Some(cond);
        match parent.kind() {
            "if_statement" if is_condition => {
                if self.python {
                    self.py_if(parent)
                } else {
                    self.js_if(parent)
                }
            }
            "ternary_expression" if is_condition => {
                let on = parent.child_by_field_name("consequence")?;
                let off = parent.child_by_field_name("alternative")?;
                let span = Self::span(parent);
                Some(Collapse { on: (span, self.operand(on)), off: (span, self.operand(off)) })
            }
            "conditional_expression" => {
                let kids = named(parent);
                if kids.len() != 3 || kids[1] != call {
                    return None;
                }
                let span = Self::span(parent);
                Some(Collapse { on: (span, self.operand(kids[0])), off: (span, self.operand(kids[2])) })
            }
            _ => None,
        }
    }

    /// A branch of a conditional expression, in parentheses when it binds
    /// looser than what can stand where the whole expression stood.
    fn operand(&self, n: Node<'_>) -> String {
        let t = self.t(n);
        if matches!(n.kind(), "sequence_expression" | "assignment_expression" | "augmented_assignment_expression" | "arrow_function" | "yield_expression" | "lambda" | "conditional_expression") {
            format!("({t})")
        } else {
            t.to_owned()
        }
    }

    fn indent(&self, at: usize) -> &'a str {
        let ls = line_start(self.text, at);
        let line = &self.text[ls..at];
        &line[..line.len() - line.trim_start().len()]
    }

    /// A JS statement kept in place of the `if`: a block's statements,
    /// re-indented; refused when the block declares a name, which would
    /// leak into the enclosing scope.
    fn js_unwrap(&self, stmt: Node<'_>, indent: &str) -> Option<String> {
        if stmt.kind() != "statement_block" {
            return Some(self.t(stmt).to_owned());
        }
        if named(stmt).iter().any(|c| matches!(c.kind(), "lexical_declaration" | "class_declaration" | "function_declaration" | "generator_function_declaration")) {
            return None;
        }
        let inner = &self.text[stmt.start_byte() + 1..stmt.end_byte() - 1];
        Some(reindent(&inner_lines(inner), indent))
    }

    fn removal(&self, stmt: Node<'_>) -> (Span, String) {
        (whole_lines(self.text, stmt.start_byte(), stmt.end_byte()), String::new())
    }

    fn js_if(&self, ifn: Node<'_>) -> Option<Collapse> {
        let cons = ifn.child_by_field_name("consequence")?;
        let alt = ifn.child_by_field_name("alternative").and_then(|e| named(e).into_iter().next());
        let span = Self::span(ifn);
        if ifn.parent().is_some_and(|p| p.kind() == "else_clause") {
            // `else if (flag) A else B` → `else A` / `else B`.
            let off = alt.map(|a| self.t(a).to_owned()).unwrap_or_else(|| "{}".to_owned());
            return Some(Collapse { on: (span, self.t(cons).to_owned()), off: (span, off) });
        }
        if !alone_before(self.text, ifn.start_byte()) {
            return None;
        }
        let indent = self.indent(ifn.start_byte());
        let keep = |stmt: Node<'_>| -> Option<(Span, String)> {
            if stmt.kind() == "if_statement" {
                return Some((span, self.t(stmt).to_owned()));
            }
            let body = self.js_unwrap(stmt, indent)?;
            Some(if body.is_empty() { self.removal(ifn) } else { (span, body) })
        };
        let on = keep(cons)?;
        let off = match alt {
            Some(a) => keep(a)?,
            None => self.removal(ifn),
        };
        Some(Collapse { on, off })
    }

    /// A Python block kept in place of the `if`, dedented to its column.
    fn py_block(&self, block: Node<'_>, indent: &str) -> String {
        let text = &self.text[block.start_byte()..block.end_byte()];
        let col = block.start_byte() - line_start(self.text, block.start_byte());
        let cut = col.saturating_sub(indent.len());
        let mut out = String::new();
        for (i, l) in text.split('\n').enumerate() {
            if i > 0 {
                out.push('\n');
                let ws = l.len() - l.trim_start().len();
                out.push_str(&l[ws.min(cut)..]);
            } else {
                out.push_str(l);
            }
        }
        out
    }

    fn py_if(&self, ifn: Node<'_>) -> Option<Collapse> {
        if !alone_before(self.text, ifn.start_byte()) {
            return None;
        }
        let indent = self.indent(ifn.start_byte());
        let span = Self::span(ifn);
        let cons = ifn.child_by_field_name("consequence")?;
        let on = (span, self.py_block(cons, indent));
        let mut c = ifn.walk();
        let first_alt = ifn.children_by_field_name("alternative", &mut c).next();
        let off = match first_alt {
            Some(e) if e.kind() == "elif_clause" => (span, format!("if{}", &self.text[e.start_byte() + 4..ifn.end_byte()])),
            Some(e) => (span, self.py_block(e.child_by_field_name("body")?, indent)),
            None => {
                let only = ifn.parent().is_some_and(|b| b.kind() == "block" && named(b).len() == 1);
                if only { (span, "pass".to_owned()) } else { self.removal(ifn) }
            }
        };
        Some(Collapse { on, off })
    }

    // ---- constants -------------------------------------------------------

    fn consts(&mut self, root: Node<'_>) {
        for mut n in named(root) {
            if n.kind() == "export_statement" {
                match n.child_by_field_name("declaration") {
                    Some(d) => n = d,
                    None => continue,
                }
            }
            match n.kind() {
                "lexical_declaration" | "variable_declaration" => {
                    for d in named(n).into_iter().filter(|d| d.kind() == "variable_declarator") {
                        let (Some(name), Some(value)) = (d.child_by_field_name("name"), d.child_by_field_name("value")) else { continue };
                        if name.kind() == "identifier" {
                            let name = self.t(name).to_owned();
                            self.const_value(&name, value, 0);
                        }
                    }
                }
                "enum_declaration" => {
                    let (Some(name), Some(body)) = (n.child_by_field_name("name"), n.child_by_field_name("body")) else { continue };
                    let name = self.t(name).to_owned();
                    for m in named(body).into_iter().filter(|m| m.kind() == "enum_assignment") {
                        let (Some(k), Some(v)) = (m.child_by_field_name("name"), m.child_by_field_name("value")) else { continue };
                        if let Some(s) = self.string(v) {
                            self.f.consts.push((format!("{name}.{}", self.t(k)), s));
                        }
                    }
                }
                "expression_statement" if self.python => {
                    if let Some(a) = named(n).into_iter().next() {
                        self.py_assignment(a, "");
                    }
                }
                "class_definition" if self.python => {
                    let (Some(name), Some(body)) = (n.child_by_field_name("name"), n.child_by_field_name("body")) else { continue };
                    let prefix = format!("{}.", self.t(name));
                    for s in named(body).into_iter().filter(|s| s.kind() == "expression_statement") {
                        if let Some(a) = named(s).into_iter().next() {
                            self.py_assignment(a, &prefix);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn py_assignment(&mut self, a: Node<'_>, prefix: &str) {
        if a.kind() != "assignment" {
            return;
        }
        let (Some(l), Some(r)) = (a.child_by_field_name("left"), a.child_by_field_name("right")) else { return };
        if l.kind() == "identifier"
            && let Some(s) = self.string(r)
        {
            self.f.consts.push((format!("{prefix}{}", self.t(l)), s));
        }
    }

    fn const_value(&mut self, name: &str, value: Node<'_>, depth: usize) {
        if depth > 4 {
            return;
        }
        if let Some(s) = self.string(value) {
            self.f.consts.push((name.to_owned(), s));
            return;
        }
        match value.kind() {
            "as_expression" | "satisfies_expression" | "parenthesized_expression" => {
                if let Some(i) = named(value).into_iter().next() {
                    self.const_value(name, i, depth + 1);
                }
            }
            "object" => {
                for p in named(value).into_iter().filter(|p| p.kind() == "pair") {
                    let (Some(k), Some(v)) = (p.child_by_field_name("key"), p.child_by_field_name("value")) else { continue };
                    let key = self.string(k).unwrap_or_else(|| self.t(k).to_owned());
                    self.const_value(&format!("{name}.{key}"), v, depth + 1);
                }
            }
            _ => {}
        }
    }

    // ---- markers ---------------------------------------------------------

    fn marker(&mut self, n: Node<'_>) {
        let text = self.t(n);
        let start = n.start_byte();
        // The comment's own words: after `//`, `#`, `/*` and a leading `*`.
        let body_at = text.find(|c: char| !matches!(c, '/' | '#' | '*' | ' ' | '\t')).unwrap_or(text.len());
        let body = &text[body_at..];
        let Some(rest) = body.strip_prefix(MARKER) else { return };
        if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
            return;
        }
        let content = rest.trim_end().trim_end_matches("*/").trim_end();
        let end = start + body_at + MARKER.len() + content.len();
        let mut m = Marker {
            span: (start + body_at, end),
            removal: whole_lines(self.text, start, n.end_byte()),
            line: line_of(self.text, start),
            ..Default::default()
        };
        // A comment after code on its line goes with the space before it,
        // and the line's end stays.
        if !alone_before(self.text, start) {
            let before = self.text[..start].trim_end_matches([' ', '\t']).len();
            m.removal = (before, n.end_byte());
        }
        let mut words = content.split_whitespace();
        match words.next() {
            Some(name) => m.name = name.to_owned(),
            None => m.bad = Some("it names no flag".into()),
        }
        while let Some(key) = words.next() {
            if key == "reason" {
                let r: Vec<&str> = words.by_ref().collect();
                m.reason = Some(r.join(" ")).filter(|r| !r.is_empty());
                break;
            }
            let Some(value) = words.next() else {
                m.bad.get_or_insert_with(|| format!("`{key}` has no value"));
                break;
            };
            let slot = match key {
                "expires" => &mut m.expires,
                "owner" => &mut m.owner,
                "ticket" => &mut m.ticket,
                "keep" => &mut m.keep,
                other => {
                    m.bad.get_or_insert_with(|| format!("`{other}` is not a marker key (expires, owner, ticket, keep, reason)"));
                    continue;
                }
            };
            *slot = Some(value.to_owned());
        }
        self.f.markers.push(m);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn js(src: &str) -> FlagFacts {
        extract_flags(Language::Typescript, src).unwrap()
    }

    #[test]
    fn a_parameter_that_flows_unchanged_is_a_param() {
        let f = js("export function isOn(name: string) { return client.variation(name, false); }\nfunction g(n) { n = n + 1; return isOn(n); }");
        assert_eq!(f.funcs[0].name, "isOn");
        assert!(f.funcs[0].top);
        assert!(matches!(f.calls[0].args[0], Arg::Param { func: 0, index: 0, .. }));
        assert!(matches!(f.calls[1].args[0], Arg::Other(_)));
    }

    #[test]
    fn constants_enums_and_objects() {
        let f = js("export const X = \"x\";\nexport const F = { A: \"a\" } as const;\nenum E { B = \"b\" }");
        assert_eq!(f.consts, vec![("X".into(), "x".into()), ("F.A".into(), "a".into()), ("E.B".into(), "b".into())]);
        let p = extract_flags(Language::Python, "NEW = \"new\"\nclass Flags(str, Enum):\n    OLD = \"old\"\n").unwrap();
        assert_eq!(p.consts, vec![("NEW".into(), "new".into()), ("Flags.OLD".into(), "old".into())]);
    }

    #[test]
    fn markers_are_read_and_bad_ones_said_why() {
        let f = js("// codelaw-flag new-checkout expires 2027-02-28 owner @team reason a long one\nx(); // codelaw-flag y keep maybe ticket\n// not a codelaw-flag marker");
        assert_eq!(f.markers.len(), 2);
        assert_eq!(f.markers[0].name, "new-checkout");
        assert_eq!(f.markers[0].expires.as_deref(), Some("2027-02-28"));
        assert_eq!(f.markers[0].reason.as_deref(), Some("a long one"));
        assert!(f.markers[1].bad.as_deref().unwrap().contains("ticket"));
    }

    #[test]
    fn collapse_keeps_one_branch() {
        let src = "function f() {\n  if (isOn(\"x\")) {\n    a();\n    b();\n  } else {\n    c();\n  }\n}\n";
        let f = js(src);
        let c = f.calls.iter().find(|c| c.callee == ["isOn"]).unwrap().collapse.clone().unwrap();
        assert_eq!(c.on.1, "a();\n  b();");
        assert_eq!(c.off.1, "c();");
        let src = "if (isOn(\"x\")) {\n  a();\n}\nz();\n";
        let c = js(src).calls[0].collapse.clone().unwrap();
        assert_eq!(&src[c.off.0.0..c.off.0.1], "if (isOn(\"x\")) {\n  a();\n}\n");
        assert_eq!(c.off.1, "");
        // A side effect in the condition: no collapse.
        assert!(js("if (isOn(next())) { a(); }").calls.iter().find(|c| c.callee == ["isOn"]).unwrap().collapse.is_none());
        // A block that declares a name: no collapse.
        assert!(js("if (isOn(\"x\")) { const a = 1; use(a); }").calls[0].collapse.is_none());
    }

    #[test]
    fn python_collapse_dedents() {
        let src = "def f():\n    if is_on(\"x\"):\n        a()\n        b()\n    else:\n        c()\n";
        let f = extract_flags(Language::Python, src).unwrap();
        let c = f.calls.iter().find(|c| c.callee == ["is_on"]).unwrap().collapse.clone().unwrap();
        assert_eq!(c.on.1, "a()\n    b()");
        assert_eq!(c.off.1, "c()");
        let src = "def f():\n    if is_on(\"x\"):\n        a()\n";
        let c = extract_flags(Language::Python, src).unwrap().calls[0].collapse.clone().unwrap();
        assert_eq!(c.off.1, "pass");
    }
}
