//! Size limits for JavaScript, TypeScript and Python (W-58): `max-lines`
//! (lines per file), `max-lines-per-function` and `max-params`, for the
//! `js/` and `py/` namespaces. Counting needs no pattern, only the parsed
//! tree: which lines hold only a comment, a docstring or an import, and which
//! nodes are functions.
//!
//! Defaults follow ESLint and Pylint so `codelaw migrate` maps them one to
//! one. A line that holds code and a comment counts as code; `{/* … */}` in
//! JSX is a comment. The rules measure the host file, never an embedded
//! region (a `<script>` in HTML, a fenced block in Markdown) for `max-lines`.

use crate::python_util::{decorator_target, decorators, is_docstring_stmt};
use crate::util::text;
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use regex::Regex;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use tree_sitter::Node;

const JS_LANGUAGES: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];
const PY_LANGUAGES: &[Language] = &[Language::Python];

/// What a size rule measures.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Measure {
    FileLines,
    FunctionLines,
    Params,
}

pub struct Size {
    pub id: &'static str,
    pub py: bool,
    pub measure: Measure,
    kinds: &'static [&'static str],
}

impl Size {
    pub const fn js(id: &'static str, measure: Measure) -> Size {
        let kinds = match measure {
            Measure::Params => JS_PARAM_KINDS,
            _ => &["program"],
        };
        Size { id, py: false, measure, kinds }
    }
    pub const fn py(id: &'static str, measure: Measure) -> Size {
        let kinds = match measure {
            Measure::Params => PY_PARAM_KINDS,
            _ => &["module"],
        };
        Size { id, py: true, measure, kinds }
    }
}

const JS_PARAM_KINDS: &[&str] = &[
    "program",
    "function_declaration",
    "generator_function_declaration",
    "function_expression",
    "generator_function",
    "arrow_function",
    "method_definition",
    "function_signature",
    "method_signature",
    "abstract_method_signature",
];
const PY_PARAM_KINDS: &[&str] = &["module", "function_definition"];

impl NativeRule for Size {
    fn id(&self) -> &'static str {
        self.id
    }
    fn languages(&self) -> &'static [Language] {
        if self.py { PY_LANGUAGES } else { JS_LANGUAGES }
    }
    fn kinds(&self) -> &'static [&'static str] {
        self.kinds
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let root = matches!(n.kind(), "program" | "module");
        if root && let Some(why) = bad_regex(ctx.options) {
            out.push(NativeFinding { start: 0, end: 0, message: why, edits: Vec::new() });
        }
        match self.measure {
            Measure::FileLines if root => file_lines(self, n, ctx, out),
            Measure::FunctionLines if root => function_lines(self, n, ctx, out),
            Measure::Params if !root => params(self, n, ctx, out),
            _ => {}
        }
    }
    /// The measured value and the limit, parsed back from the message the
    /// rule wrote, for `--format json` and SARIF.
    fn data_of(&self, finding: &NativeFinding, _options: &BTreeMap<String, Value>) -> Option<Value> {
        let (head, tail) = finding.message.rsplit_once(" (max ")?;
        let max: u64 = tail.chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok()?;
        let (rest, key) = [(" positional parameters", "positional"), (" parameters", "parameters"), (" parameter", "parameters"), (" lines", "lines"), (" line", "lines")]
            .iter()
            .find_map(|(suffix, key)| head.strip_suffix(suffix).map(|r| (r, *key)))?;
        let digits: String = rest.chars().rev().take_while(char::is_ascii_digit).collect::<Vec<_>>().into_iter().rev().collect();
        let measured: u64 = digits.parse().ok()?;
        Some(json!({ key: measured, "max": max }))
    }
}

// ---- options ----

fn uint(o: &BTreeMap<String, Value>, key: &str, default: usize) -> usize {
    o.get(key).and_then(Value::as_u64).map_or(default, |v| v as usize)
}

fn opt_uint(o: &BTreeMap<String, Value>, key: &str) -> Option<usize> {
    o.get(key).and_then(Value::as_u64).map(|v| v as usize)
}

fn flag(o: &BTreeMap<String, Value>, key: &str, default: bool) -> bool {
    o.get(key).and_then(Value::as_bool).unwrap_or(default)
}

thread_local! {
    static REGEXES: RefCell<HashMap<String, Option<Regex>>> = RefCell::new(HashMap::new());
}

/// Whether `name` matches the regular expression option `key`; a missing or
/// invalid expression matches nothing (the invalid one is reported once per
/// file by `bad_regex`).
fn name_matches(o: &BTreeMap<String, Value>, key: &str, name: &str) -> bool {
    let Some(pattern) = o.get(key).and_then(Value::as_str) else { return false };
    REGEXES.with(|c| c.borrow_mut().entry(pattern.to_owned()).or_insert_with(|| Regex::new(pattern).ok()).as_ref().is_some_and(|r| r.is_match(name)))
}

fn bad_regex(o: &BTreeMap<String, Value>) -> Option<String> {
    let pattern = o.get("ignore_names").and_then(Value::as_str)?;
    Regex::new(pattern).err().map(|e| format!("option `ignore_names` is not a valid regular expression: {e}"))
}

// ---- line classes ----

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Blank,
    Comment,
    Docstring,
    Import,
    Code,
}

/// Which line classes are not counted.
struct Skips {
    blank: bool,
    comment: bool,
    docstring: bool,
    import: bool,
}

impl Skips {
    fn read(o: &BTreeMap<String, Value>, docstring_default: bool) -> Skips {
        Skips { blank: flag(o, "skip_blank_lines", true), comment: flag(o, "skip_comments", true), docstring: flag(o, "skip_docstrings", docstring_default), import: flag(o, "skip_imports", false) }
    }
    fn counts(&self, c: Class) -> bool {
        match c {
            Class::Blank => !self.blank,
            Class::Comment => !self.comment,
            Class::Docstring => !self.docstring,
            Class::Import => !self.import,
            Class::Code => true,
        }
    }
    /// "blank lines, comments and docstrings not counted".
    fn note(&self) -> String {
        let mut parts = Vec::new();
        for (on, name) in [(self.blank, "blank lines"), (self.comment, "comments"), (self.docstring, "docstrings"), (self.import, "imports")] {
            if on {
                parts.push(name);
            }
        }
        match parts.len() {
            0 => String::new(),
            1 => format!("{} not counted", parts[0]),
            _ => format!("{} and {} not counted", parts[..parts.len() - 1].join(", "), parts[parts.len() - 1]),
        }
    }
}

/// Ranges of bytes that are a comment, a docstring or an import, in source
/// order and not overlapping.
fn special_ranges(root: Node<'_>, py: bool) -> Vec<(usize, usize, Class)> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        let class = match n.kind() {
            "comment" => Some(Class::Comment),
            "jsx_expression" if !py && n.named_child_count() == 1 && n.named_child(0).is_some_and(|c| c.kind() == "comment") => Some(Class::Comment),
            "expression_statement" if py && is_docstring(n) => Some(Class::Docstring),
            "import_statement" | "import_from_statement" | "future_import_statement" if n.parent().is_some_and(|p| matches!(p.kind(), "program" | "module")) => Some(Class::Import),
            _ => None,
        };
        if let Some(c) = class {
            out.push((n.start_byte(), n.end_byte(), c));
            continue;
        }
        if n.child_count() > 0 {
            let mut cur = n.walk();
            let kids: Vec<Node<'_>> = n.children(&mut cur).collect();
            stack.extend(kids.into_iter().rev());
        }
    }
    out
}

/// A docstring: the first statement of a module, class or function body.
fn is_docstring(n: Node<'_>) -> bool {
    if !is_docstring_stmt(n) {
        return false;
    }
    let Some(parent) = n.parent() else { return false };
    let owner_ok = match parent.kind() {
        "module" => true,
        "block" => parent.parent().is_some_and(|p| matches!(p.kind(), "function_definition" | "class_definition")),
        _ => false,
    };
    let mut cur = parent.walk();
    let first = parent.named_children(&mut cur).find(|c| c.kind() != "comment");
    owner_ok && first.is_some_and(|f| f.id() == n.id())
}

/// The class of each line of `src`.
fn classify(src: &str, ranges: &[(usize, usize, Class)]) -> Vec<Class> {
    let bytes = src.as_bytes();
    let mut classes = Vec::new();
    let mut ri = 0;
    let mut start = 0;
    while start < bytes.len() {
        let end = bytes[start..].iter().position(|&b| b == b'\n').map_or(bytes.len(), |p| start + p);
        let (mut code, mut comment, mut doc, mut import) = (false, false, false, false);
        let mut pos = start;
        while pos < end {
            if matches!(bytes[pos], b' ' | b'\t' | b'\r' | 0x0c) {
                pos += 1;
                continue;
            }
            while ri < ranges.len() && ranges[ri].1 <= pos {
                ri += 1;
            }
            match ranges.get(ri) {
                Some(&(s, e, c)) if s <= pos => {
                    match c {
                        Class::Comment => comment = true,
                        Class::Docstring => doc = true,
                        Class::Import => import = true,
                        _ => {}
                    }
                    pos = e.min(end).max(pos + 1);
                }
                _ => {
                    code = true;
                    pos = end;
                }
            }
        }
        classes.push(if code {
            Class::Code
        } else if import {
            Class::Import
        } else if doc {
            Class::Docstring
        } else if comment {
            Class::Comment
        } else {
            Class::Blank
        });
        start = end + 1;
    }
    classes
}

/// Counted lines before each line: `prefix[i]` counts lines `0..i`.
fn prefix_counts(classes: &[Class], skips: &Skips) -> Vec<usize> {
    let mut prefix = Vec::with_capacity(classes.len() + 1);
    prefix.push(0);
    for &c in classes {
        prefix.push(prefix.last().copied().unwrap_or(0) + usize::from(skips.counts(c)));
    }
    prefix
}

struct Measured {
    prefix: Vec<usize>,
}

impl Measured {
    fn new(root: Node<'_>, src: &str, py: bool, skips: &Skips) -> Measured {
        let ranges = special_ranges(root, py);
        Measured { prefix: prefix_counts(&classify(src, &ranges), skips) }
    }
    /// Counted lines in rows `first..=last`.
    fn rows(&self, first: usize, last: usize) -> usize {
        let last = (last + 1).min(self.prefix.len() - 1);
        let first = first.min(last);
        self.prefix[last] - self.prefix[first]
    }
}

// ---- max-lines ----

/// A host file's embedded region is not a file.
fn is_host_region(ctx: &NativeContext<'_>) -> bool {
    let path = ctx.file.path();
    let ext = path.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    matches!(ext.as_deref(), Some("html" | "htm" | "xhtml" | "md" | "markdown" | "mdx"))
}

fn file_lines(rule: &Size, root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if is_host_region(ctx) {
        return;
    }
    let o = ctx.options;
    let max = uint(o, "max", if rule.py { 1000 } else { 300 });
    let skips = Skips::read(o, false);
    let m = Measured::new(root, ctx.source, rule.py, &skips);
    let total = m.prefix.last().copied().unwrap_or(0);
    if total <= max {
        return;
    }
    // The first line over the limit: the one that makes the count `max + 1`.
    let row = m.prefix.partition_point(|&c| c < max + 1) - 1;
    let (start, end) = line_span(ctx.source, row);
    let note = skips.note();
    let note = if note.is_empty() { String::new() } else { format!("; {note}") };
    out.push(NativeFinding { start, end, message: format!("{total} lines (max {max}{note})"), edits: Vec::new() });
}

/// The bytes of line `row` without its indent and line break.
fn line_span(src: &str, row: usize) -> (usize, usize) {
    let mut start = 0;
    for _ in 0..row {
        match src[start..].find('\n') {
            Some(p) => start += p + 1,
            None => break,
        }
    }
    let end = src[start..].find('\n').map_or(src.len(), |p| start + p);
    let line = &src[start..end];
    let trimmed_start = start + (line.len() - line.trim_start().len());
    (trimmed_start, start + line.trim_end().len().max(trimmed_start - start))
}

// ---- functions ----

fn is_function(kind: &str, py: bool) -> bool {
    if py {
        return kind == "function_definition";
    }
    matches!(
        kind,
        "function_declaration"
            | "generator_function_declaration"
            | "function_expression"
            | "generator_function"
            | "arrow_function"
            | "method_definition"
            | "class_static_block"
    )
}

/// `(() => { … })()` and `(function () { … })()`.
fn is_iife(n: Node<'_>) -> bool {
    if !matches!(n.kind(), "function_expression" | "generator_function" | "arrow_function") {
        return false;
    }
    let mut cur = n;
    while let Some(p) = cur.parent() {
        match p.kind() {
            "parenthesized_expression" => cur = p,
            "call_expression" => return p.child_by_field_name("function").is_some_and(|f| f.id() == cur.id()),
            _ => return false,
        }
    }
    false
}

/// The node that names a function, and the name.
fn name_of<'t>(n: Node<'t>, src: &str) -> (Option<Node<'t>>, Option<String>) {
    if n.kind() == "class_static_block" {
        return (None, Some("static block".to_owned()));
    }
    if let Some(name) = n.child_by_field_name("name") {
        return (Some(name), Some(text(name, src).to_owned()));
    }
    let mut cur = n;
    while let Some(p) = cur.parent() {
        match p.kind() {
            "parenthesized_expression" | "as_expression" | "satisfies_expression" | "non_null_expression" => cur = p,
            "variable_declarator" => {
                let name = p.child_by_field_name("name").filter(|c| c.kind() == "identifier");
                return (name, name.map(|c| text(c, src).to_owned()));
            }
            "assignment_expression" => {
                let left = p.child_by_field_name("left");
                return (left, left.map(|c| text(c, src).to_owned()));
            }
            "pair" => {
                let key = p.child_by_field_name("key");
                return (key, key.map(|c| text(c, src).trim_matches(|ch| ch == '"' || ch == '\'').to_owned()));
            }
            "field_definition" | "public_field_definition" => {
                let name = p.child_by_field_name("property").or_else(|| p.child_by_field_name("name"));
                return (name, name.map(|c| text(c, src).to_owned()));
            }
            _ => break,
        }
    }
    (None, None)
}

/// Where a finding about function `n` is reported: its name, or the first
/// line of the function.
fn function_span(n: Node<'_>, name: Option<Node<'_>>, src: &str) -> (usize, usize) {
    match name {
        Some(c) => (c.start_byte(), c.end_byte()),
        None => {
            let end = src[n.start_byte()..].find('\n').map_or(src.len(), |p| n.start_byte() + p).min(n.end_byte());
            (n.start_byte(), end)
        }
    }
}

fn function_lines(rule: &Size, root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let o = ctx.options;
    let max = uint(o, "max", 50);
    let skips = Skips::read(o, rule.py);
    let nested = flag(o, "include_nested", true);
    let iife = flag(o, "include_iife", false);
    let m = Measured::new(root, ctx.source, rule.py, &skips);
    let note = skips.note();
    let note = if note.is_empty() { String::new() } else { format!("; {note}") };
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if is_function(n.kind(), rule.py) && (iife || !is_iife(n)) {
            let (name_node, name) = name_of(n, ctx.source);
            let exempt = name.as_deref().is_some_and(|s| name_matches(o, "ignore_names", s));
            if !exempt {
                let (first, last) = rows(n);
                let mut count = m.rows(first, last);
                if !nested {
                    for (s, e) in nested_rows(n, rule.py, iife, first, last) {
                        count = count.saturating_sub(m.rows(s, e));
                    }
                }
                if count > max {
                    let (start, end) = function_span(n, name_node, ctx.source);
                    let who = name.map_or_else(|| "Anonymous function".to_owned(), |s| format!("`{s}`"));
                    out.push(NativeFinding { start, end, message: format!("{who} is {count} lines (max {max}{note})"), edits: Vec::new() });
                }
            }
        }
        if n.child_count() > 0 {
            let mut cur = n.walk();
            let kids: Vec<Node<'_>> = n.children(&mut cur).collect();
            stack.extend(kids.into_iter().rev());
        }
    }
}

/// The rows a node spans: a node ending at column 0 does not own that row.
fn rows(n: Node<'_>) -> (usize, usize) {
    let first = n.start_position().row;
    let end = n.end_position();
    let last = if end.column == 0 && end.row > first { end.row - 1 } else { end.row };
    (first, last)
}

/// The row ranges of the outermost functions inside `n`, merged, within
/// `first..=last`.
fn nested_rows(n: Node<'_>, py: bool, iife: bool, first: usize, last: usize) -> Vec<(usize, usize)> {
    let mut found: Vec<(usize, usize)> = Vec::new();
    let mut stack: Vec<Node<'_>> = Vec::new();
    let mut cur = n.walk();
    stack.extend(n.children(&mut cur).collect::<Vec<_>>().into_iter().rev());
    while let Some(c) = stack.pop() {
        if is_function(c.kind(), py) && (iife || !is_iife(c)) {
            let (s, e) = rows(c);
            found.push((s.max(first), e.min(last)));
            continue;
        }
        if c.child_count() > 0 {
            let mut cur = c.walk();
            let kids: Vec<Node<'_>> = c.children(&mut cur).collect();
            stack.extend(kids.into_iter().rev());
        }
    }
    found.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in found {
        match merged.last_mut() {
            Some(l) if s <= l.1 => l.1 = l.1.max(e),
            _ => merged.push((s, e)),
        }
    }
    merged
}

// ---- parameters ----

/// One parameter, seen the same way in JS, TS and Python.
struct Param {
    name: Option<String>,
    /// `...rest`, `*args`, `**kwargs`.
    varargs: bool,
    /// Before any `*` or `*args` (Python).
    positional: bool,
    /// `this: T` (TS).
    this: bool,
    /// `private a: T`, `readonly b` (TS).
    property: bool,
}

fn params(rule: &Size, n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let o = ctx.options;
    let src = ctx.source;
    let kind = n.kind();
    let max = uint(o, "max", if rule.py { 5 } else { 3 });
    let list = if rule.py { py_params(n, src) } else { js_params(n, src) };
    let (name_node, name) = name_of(n, src);

    if flag(o, "ignore_declarations", true) && is_declaration(n, kind, ctx) {
        return;
    }
    if flag(o, "ignore_overrides", true) && is_override(n, rule.py, src) {
        return;
    }
    let count_this = flag(o, "count_this", false);
    let count_varargs = flag(o, "count_varargs", false);
    let ignore_self = rule.py && flag(o, "ignore_self_cls", true) && in_class(n);
    let mut counted: Vec<&Param> = Vec::new();
    for (i, p) in list.iter().enumerate() {
        if p.this && !count_this {
            continue;
        }
        if p.varargs && !count_varargs {
            continue;
        }
        if ignore_self && i == 0 && p.name.as_deref().is_some_and(|s| s == "self" || s == "cls") {
            continue;
        }
        if p.name.as_deref().is_some_and(|s| name_matches(o, "ignore_names", s)) {
            continue;
        }
        counted.push(p);
    }
    if !rule.py && flag(o, "ignore_constructor_injection", false) && name.as_deref() == Some("constructor") && !counted.is_empty() && counted.iter().all(|p| p.property) {
        return;
    }
    let who = |name: &Option<String>| name.as_ref().map_or_else(|| "Anonymous function".to_owned(), |s| format!("`{s}`"));
    let (start, end) = function_span(n, name_node, src);
    if counted.len() > max {
        let noun = if counted.len() == 1 { "parameter" } else { "parameters" };
        out.push(NativeFinding { start, end, message: format!("{} takes {} {noun} (max {max})", who(&name), counted.len()), edits: Vec::new() });
    }
    if rule.py
        && let Some(max_pos) = opt_uint(o, "max_positional")
    {
        let positional = counted.iter().filter(|p| p.positional && !p.varargs).count();
        if positional > max_pos {
            let noun = if positional == 1 { "positional parameter" } else { "positional parameters" };
            out.push(NativeFinding { start, end, message: format!("{} takes {positional} {noun} (max {max_pos})", who(&name)), edits: Vec::new() });
        }
    }
}

fn is_declaration(n: Node<'_>, kind: &str, ctx: &NativeContext<'_>) -> bool {
    if matches!(kind, "function_signature" | "method_signature" | "abstract_method_signature") {
        return true;
    }
    if ctx.file.path().ends_with(".d.ts") {
        return true;
    }
    if ctx.language == Language::Python {
        return decorators(n).iter().any(|d| decorator_name(*d, ctx.source) == "overload");
    }
    let mut p = n.parent();
    while let Some(x) = p {
        if x.kind() == "ambient_declaration" {
            return true;
        }
        p = x.parent();
    }
    false
}

fn is_override(n: Node<'_>, py: bool, src: &str) -> bool {
    if py {
        return decorators(n).iter().any(|d| decorator_name(*d, src) == "override");
    }
    let mut cur = n.walk();
    n.children(&mut cur).any(|c| c.kind() == "override_modifier")
}

/// The last segment of a decorator's name: `@typing.override` is `override`.
fn decorator_name(d: Node<'_>, src: &str) -> String {
    let target = decorator_target(d);
    text(target, src).rsplit('.').next().unwrap_or("").to_owned()
}

fn in_class(def: Node<'_>) -> bool {
    let mut p = def.parent();
    if p.is_some_and(|x| x.kind() == "decorated_definition") {
        p = p.and_then(|x| x.parent());
    }
    p.is_some_and(|b| b.kind() == "block" && b.parent().is_some_and(|c| c.kind() == "class_definition"))
}

fn js_params(n: Node<'_>, src: &str) -> Vec<Param> {
    let mut list = Vec::new();
    let items: Vec<Node<'_>> = match n.child_by_field_name("parameters") {
        Some(ps) => {
            let mut cur = ps.walk();
            ps.named_children(&mut cur).filter(|c| c.kind() != "comment").collect()
        }
        None => n.child_by_field_name("parameter").into_iter().collect(),
    };
    for p in items {
        let pattern = if matches!(p.kind(), "required_parameter" | "optional_parameter") { p.child_by_field_name("pattern").unwrap_or(p) } else { p };
        let mut cur = p.walk();
        let property = p.children(&mut cur).any(|c| matches!(c.kind(), "accessibility_modifier" | "readonly" | "override_modifier"));
        let (name, varargs, this) = match pattern.kind() {
            "this" => (None, false, true),
            "rest_pattern" => (None, true, false),
            "identifier" => (Some(text(pattern, src).to_owned()), false, text(pattern, src) == "this"),
            "assignment_pattern" => (pattern.child_by_field_name("left").filter(|l| l.kind() == "identifier").map(|l| text(l, src).to_owned()), false, false),
            _ => (None, false, false),
        };
        list.push(Param { name, varargs, positional: true, this, property });
    }
    list
}

fn py_params(n: Node<'_>, src: &str) -> Vec<Param> {
    let mut list = Vec::new();
    let Some(ps) = n.child_by_field_name("parameters") else { return list };
    let mut starred = false;
    let mut cur = ps.walk();
    for p in ps.named_children(&mut cur) {
        let mut inner = p;
        if p.kind() == "typed_parameter"
            && let Some(first) = p.named_child(0)
        {
            inner = first;
        }
        match inner.kind() {
            "positional_separator" | "comment" => {}
            "keyword_separator" => starred = true,
            "list_splat_pattern" => {
                starred = true;
                list.push(Param { name: None, varargs: true, positional: false, this: false, property: false });
            }
            "dictionary_splat_pattern" => list.push(Param { name: None, varargs: true, positional: false, this: false, property: false }),
            _ => {
                let name_node = if matches!(inner.kind(), "default_parameter" | "typed_default_parameter") { inner.child_by_field_name("name") } else { Some(inner) };
                let name = name_node.filter(|c| c.kind() == "identifier").map(|c| text(c, src).to_owned());
                list.push(Param { name, varargs: false, positional: !starred, this: false, property: false });
            }
        }
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DetachedFile;

    const JS_FILE: Size = Size::js("js/max-lines", Measure::FileLines);
    const JS_FN: Size = Size::js("js/max-lines-per-function", Measure::FunctionLines);
    const JS_PARAMS: Size = Size::js("js/max-params", Measure::Params);
    const PY_FILE: Size = Size::py("py/max-lines", Measure::FileLines);
    const PY_FN: Size = Size::py("py/max-lines-per-function", Measure::FunctionLines);
    const PY_PARAMS: Size = Size::py("py/max-params", Measure::Params);

    fn run(rule: &Size, language: Language, path: &str, src: &str, options: Value) -> Vec<NativeFinding> {
        let root = codelaw_lang::parse(language, src);
        let options: BTreeMap<String, Value> = serde_json::from_value(options).unwrap();
        let data = BTreeMap::new();
        let file = DetachedFile::new(path, src);
        let ctx = NativeContext { source: src, language, semantic: None, options: &options, data: &data, file: &file };
        let mut out = Vec::new();
        for node in root.root().dfs() {
            if rule.kinds().contains(&node.kind().as_ref()) {
                rule.check(node, &ctx, &mut out);
            }
        }
        out
    }

    fn messages(f: &[NativeFinding]) -> Vec<&str> {
        f.iter().map(|f| f.message.as_str()).collect()
    }

    fn js(rule: &Size, src: &str, options: Value) -> Vec<NativeFinding> {
        run(rule, Language::Javascript, "a.js", src, options)
    }
    fn py(rule: &Size, src: &str, options: Value) -> Vec<NativeFinding> {
        run(rule, Language::Python, "a.py", src, options)
    }

    #[test]
    fn file_lines_skip_blank_and_comments() {
        let src = "a();\n\n// c\n/* x\n y */\nb();\nc(); // t\n";
        assert!(js(&JS_FILE, src, json!({ "max": 3 })).is_empty());
        let f = js(&JS_FILE, src, json!({ "max": 2 }));
        assert_eq!(messages(&f), ["3 lines (max 2; blank lines and comments not counted)"]);
        // reported at the first line over the limit: `c();`
        assert_eq!(&src[f[0].start..f[0].end], "c(); // t");
        let f = js(&JS_FILE, src, json!({ "max": 5, "skip_blank_lines": false, "skip_comments": false }));
        assert_eq!(messages(&f), ["7 lines (max 5)"]);
        assert_eq!(&src[f[0].start..f[0].end], "b();");
    }

    #[test]
    fn file_lines_jsx_comment_and_imports() {
        let src = "import a from 'a';\nimport b from 'b';\nx = <div>\n  {/* hi */}\n</div>;\n";
        let f = run(&JS_FILE, Language::Jsx, "a.jsx", src, json!({ "max": 2 }));
        assert_eq!(messages(&f), ["4 lines (max 2; blank lines and comments not counted)"]);
        let f = run(&JS_FILE, Language::Jsx, "a.jsx", src, json!({ "max": 2, "skip_imports": true }));
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn file_lines_python_docstrings() {
        let src = "\"\"\"Module\ndoc\"\"\"\nimport os\n\ndef f():\n    \"\"\"d\"\"\"\n    return 1\n";
        assert!(py(&PY_FILE, src, json!({ "max": 6 })).is_empty());
        let f = py(&PY_FILE, src, json!({ "max": 5 }));
        assert_eq!(messages(&f), ["6 lines (max 5; blank lines and comments not counted)"]);
        let f = py(&PY_FILE, src, json!({ "max": 2, "skip_docstrings": true }));
        assert_eq!(messages(&f), ["3 lines (max 2; blank lines, comments and docstrings not counted)"]);
    }

    #[test]
    fn host_region_is_not_a_file() {
        let src = "a();\nb();\nc();\n";
        assert!(run(&JS_FILE, Language::Javascript, "page.html", src, json!({ "max": 1 })).is_empty());
        assert_eq!(run(&JS_FILE, Language::Javascript, "page.js", src, json!({ "max": 1 })).len(), 1);
    }

    #[test]
    fn function_lines_every_form() {
        let src = "function a() {\n  1;\n  2;\n}\nconst b = () => {\n  1;\n  2;\n};\nconst o = {\n  m() {\n    1;\n    2;\n  },\n  get g() {\n    1;\n    2;\n  },\n};\nclass C {\n  static {\n    1;\n    2;\n  }\n  k() {\n    1;\n    2;\n  }\n}\nconst e = function () {\n  1;\n  2;\n};\n";
        let f = js(&JS_FN, src, json!({ "max": 3 }));
        let names: Vec<&str> = f.iter().map(|f| f.message.split(' ').next().unwrap()).collect();
        assert_eq!(names, ["`a`", "`b`", "`m`", "`g`", "`static", "`k`", "`e`"], "{:?}", messages(&f));
        assert_eq!(f[0].message, "`a` is 4 lines (max 3; blank lines and comments not counted)");
        assert_eq!(&src[f[1].start..f[1].end], "b");
    }

    #[test]
    fn function_lines_nested_and_iife() {
        let src = "function outer() {\n  const inner = () => {\n    1;\n    2;\n  };\n  inner();\n}\n";
        let f = js(&JS_FN, src, json!({ "max": 4 }));
        assert_eq!(messages(&f), ["`outer` is 7 lines (max 4; blank lines and comments not counted)"]);
        let f = js(&JS_FN, src, json!({ "max": 2, "include_nested": false }));
        assert_eq!(messages(&f), ["`outer` is 3 lines (max 2; blank lines and comments not counted)", "`inner` is 4 lines (max 2; blank lines and comments not counted)"]);
        let iife = "(() => {\n  1;\n  2;\n  3;\n})();\n";
        assert!(js(&JS_FN, iife, json!({ "max": 2 })).is_empty());
        assert_eq!(js(&JS_FN, iife, json!({ "max": 2, "include_iife": true })).len(), 1);
    }

    #[test]
    fn function_lines_ignore_names() {
        let src = "describe('x', () => {\n  1;\n  2;\n});\nfunction test_a() {\n  1;\n  2;\n}\nfunction other() {\n  1;\n  2;\n}\n";
        let f = js(&JS_FN, src, json!({ "max": 2, "ignore_names": "^test" }));
        assert_eq!(messages(&f).len(), 2, "{f:?}");
        let f = js(&JS_FN, src, json!({ "max": 2, "ignore_names": "^(test|other)" }));
        assert_eq!(messages(&f), ["Anonymous function is 4 lines (max 2; blank lines and comments not counted)"]);
        let f = js(&JS_FN, src, json!({ "max": 2, "ignore_names": "(" }));
        assert!(f[0].message.starts_with("option `ignore_names` is not a valid regular expression"), "{f:?}");
    }

    #[test]
    fn python_function_lines() {
        let src = "def f(a):\n    \"\"\"doc\n    more\"\"\"\n    # c\n\n    x = 1\n    y = 2\n\nasync def g():\n    pass\n";
        let f = py(&PY_FN, src, json!({ "max": 2 }));
        assert_eq!(messages(&f), ["`f` is 3 lines (max 2; blank lines, comments and docstrings not counted)"]);
        let f = py(&PY_FN, src, json!({ "max": 4, "skip_docstrings": false }));
        assert_eq!(messages(&f), ["`f` is 5 lines (max 4; blank lines and comments not counted)"]);
        let f = py(&PY_FN, "class A:\n    def m(self):\n        a = 1\n        b = 2\n", json!({ "max": 2 }));
        assert_eq!(messages(&f), ["`m` is 3 lines (max 2; blank lines, comments and docstrings not counted)"]);
    }

    #[test]
    fn js_params() {
        let src = "function a(x, y, z, w) {}\nconst b = (x, { y, z }, [w]) => 1;\nconst c = x => x;\nfunction d(_a, _b, _c, k) {}\nfunction e(x, y, ...r) {}\n";
        let f = js(&JS_PARAMS, src, json!({}));
        assert_eq!(messages(&f), ["`a` takes 4 parameters (max 3)", "`d` takes 4 parameters (max 3)"]);
        let f = js(&JS_PARAMS, src, json!({ "max": 2, "ignore_names": "^_" }));
        assert_eq!(messages(&f), ["`a` takes 4 parameters (max 2)", "`b` takes 3 parameters (max 2)"]);
        let f = js(&JS_PARAMS, src, json!({ "max": 2, "count_varargs": true }));
        assert!(messages(&f).contains(&"`e` takes 3 parameters (max 2)"), "{f:?}");
        let f = js(&JS_PARAMS, "const f = (x) => 1;", json!({ "max": 0 }));
        assert_eq!(messages(&f), ["`f` takes 1 parameter (max 0)"]);
    }

    #[test]
    fn ts_params() {
        let src = "function f(this: Foo, a: number, b?: string, c = 1) {}\nclass A {\n  constructor(private a: number, readonly b: number, public c: number, d: number) {}\n  override m(a, b, c, d) {}\n  abstract n(a, b, c, d): void;\n  o(a, b, c, d): void;\n  o(a, b, c, d) {}\n}\ndeclare function g(a, b, c, d): void;\n";
        let ts = |opts: Value| run(&JS_PARAMS, Language::Typescript, "a.ts", src, opts);
        assert!(ts(json!({})).iter().any(|f| f.message.starts_with("`o` takes 4")), "{:?}", ts(json!({})));
        let f = ts(json!({}));
        assert_eq!(messages(&f), ["`constructor` takes 4 parameters (max 3)", "`o` takes 4 parameters (max 3)"]);
        let f = ts(json!({ "max": 3, "count_this": true }));
        assert!(messages(&f).contains(&"`f` takes 4 parameters (max 3)"), "{f:?}");
        let f = ts(json!({ "ignore_constructor_injection": true }));
        assert_eq!(messages(&f), ["`constructor` takes 4 parameters (max 3)", "`o` takes 4 parameters (max 3)"]);
        let f = ts(json!({ "ignore_overrides": false, "ignore_declarations": false }));
        assert_eq!(f.len(), 6, "{:?}", messages(&f));
        let dts = run(&JS_PARAMS, Language::Typescript, "a.d.ts", "function a(x, y, z, w);", json!({}));
        assert!(dts.is_empty());
    }

    #[test]
    fn python_params() {
        let src = "def f(a, b, c, d, e, g): pass\nclass A:\n    def m(self, a, b, c, d, e): pass\n    @classmethod\n    def k(cls, a, b, c, d, e): pass\ndef v(a, *args, **kw): pass\n";
        let f = py(&PY_PARAMS, src, json!({}));
        assert_eq!(messages(&f), ["`f` takes 6 parameters (max 5)"]);
        let f = py(&PY_PARAMS, src, json!({ "ignore_self_cls": false, "max": 5 }));
        assert_eq!(f.len(), 3, "{:?}", messages(&f));
        let f = py(&PY_PARAMS, src, json!({ "count_varargs": true, "max": 2 }));
        assert!(messages(&f).contains(&"`v` takes 3 parameters (max 2)"), "{f:?}");
        let f = py(&PY_PARAMS, "def f(a, b, c, *, d, e): pass\n", json!({ "max": 10, "max_positional": 2 }));
        assert_eq!(messages(&f), ["`f` takes 3 positional parameters (max 2)"]);
        let f = py(&PY_PARAMS, "@typing.overload\ndef f(a, b, c, d, e, g): ...\n@override\ndef h(a, b, c, d, e, g): ...\n", json!({}));
        assert!(f.is_empty(), "{f:?}");
        let f = py(&PY_PARAMS, "@overload\ndef f(a, b, c, d, e, g): ...\n@override\ndef h(a, b, c, d, e, g): ...\n", json!({ "ignore_overrides": false, "ignore_declarations": false }));
        assert_eq!(f.len(), 2);
    }

    #[test]
    fn data_carries_the_value_and_the_limit() {
        let rule = &JS_FILE;
        let f = NativeFinding { start: 0, end: 0, message: "412 lines (max 300; blank lines and comments not counted)".into(), edits: Vec::new() };
        assert_eq!(rule.data_of(&f, &BTreeMap::new()), Some(json!({ "lines": 412, "max": 300 })));
        let f = NativeFinding { start: 0, end: 0, message: "`fn2` takes 6 parameters (max 3)".into(), edits: Vec::new() };
        assert_eq!(JS_PARAMS.data_of(&f, &BTreeMap::new()), Some(json!({ "parameters": 6, "max": 3 })));
        let f = NativeFinding { start: 0, end: 0, message: "`f` takes 3 positional parameters (max 2)".into(), edits: Vec::new() };
        assert_eq!(PY_PARAMS.data_of(&f, &BTreeMap::new()), Some(json!({ "positional": 3, "max": 2 })));
        assert_eq!(PY_FN.data_of(&NativeFinding { start: 0, end: 0, message: "x".into(), edits: Vec::new() }, &BTreeMap::new()), None);
    }
}
