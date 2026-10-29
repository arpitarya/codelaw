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
    /// Cyclomatic complexity of a function (W-132).
    Complexity,
    /// How deeply blocks nest in a function (W-132).
    Depth,
    /// Lines of a React component (W-132).
    ComponentLines,
    /// Props a React component takes (W-132).
    Props,
    /// How deeply JSX elements nest (W-132).
    JsxDepth,
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
    /// A React rule: JS and TS files, a measure of a component (W-132).
    pub const fn react(id: &'static str, measure: Measure) -> Size {
        Size { id, py: false, measure, kinds: &["program"] }
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
            Measure::Complexity if root => complexity(self, n, ctx, out),
            Measure::Depth if root => depth(self, n, ctx, out),
            Measure::ComponentLines if root => component_lines(n, ctx, out),
            Measure::Props if root => component_props(n, ctx, out),
            Measure::JsxDepth if root => jsx_depth(n, ctx, out),
            _ => {}
        }
    }
    /// The measured value and the limit, parsed back from the message the
    /// rule wrote, for `--format json` and SARIF.
    fn data_of(&self, finding: &NativeFinding, _options: &BTreeMap<String, Value>) -> Option<Value> {
        let (head, tail) = finding.message.rsplit_once(" (max ")?;
        let max: u64 = tail.chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok()?;
        if let Some((_, n)) = head.split_once(" has a complexity of ") {
            return Some(json!({ "complexity": n.parse::<u64>().ok()?, "max": max }));
        }
        let (rest, key) = [(" positional parameters", "positional"), (" parameters", "parameters"), (" parameter", "parameters"), (" deep", "depth"), (" props", "props"), (" prop", "props"), (" lines", "lines"), (" line", "lines")]
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

// ---- complexity (W-132) ----

/// Whether `n` adds a branch to a function's cyclomatic complexity.
fn is_branch(n: Node<'_>, py: bool, src: &str, count_optional_chain: bool) -> bool {
    let kind = n.kind();
    if py {
        return match kind {
            "if_statement" | "elif_clause" | "for_statement" | "while_statement" | "except_clause" | "except_group_clause" | "conditional_expression" | "boolean_operator" | "for_in_clause" | "if_clause" => true,
            // `case _:` is the fall-through, as `default:` is.
            "case_clause" => n.named_child(0).is_none_or(|p| text(p, src).trim() != "_"),
            _ => false,
        };
    }
    match kind {
        "if_statement" | "ternary_expression" | "for_statement" | "for_in_statement" | "while_statement" | "do_statement" | "switch_case" | "catch_clause" => true,
        "binary_expression" => n.child_by_field_name("operator").is_some_and(|o| matches!(o.kind(), "&&" | "||" | "??")),
        "augmented_assignment_expression" => n.child_by_field_name("operator").is_some_and(|o| matches!(o.kind(), "&&=" | "||=" | "??=")),
        "optional_chain" => count_optional_chain,
        _ => false,
    }
}

/// The nodes of function `f` that are not inside a function nested in it.
fn own_nodes<'t>(f: Node<'t>, py: bool) -> Vec<Node<'t>> {
    let mut out = Vec::new();
    let mut cur = f.walk();
    let mut stack: Vec<Node<'t>> = f.children(&mut cur).collect();
    while let Some(x) = stack.pop() {
        if is_function(x.kind(), py) {
            continue;
        }
        out.push(x);
        let mut c = x.walk();
        stack.extend(x.children(&mut c).collect::<Vec<_>>());
    }
    out
}

fn complexity(rule: &Size, root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let o = ctx.options;
    let max = uint(o, "max", 20);
    let optional = flag(o, "count_optional_chain", false);
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if is_function(n.kind(), rule.py) {
            let (name_node, name) = name_of(n, ctx.source);
            let exempt = name.as_deref().is_some_and(|s| name_matches(o, "ignore_names", s));
            if !exempt {
                let count = 1 + own_nodes(n, rule.py).into_iter().filter(|x| is_branch(*x, rule.py, ctx.source, optional)).count();
                if count > max {
                    let (start, end) = function_span(n, name_node, ctx.source);
                    let who = name.map_or_else(|| "Anonymous function".to_owned(), |s| format!("`{s}`"));
                    out.push(NativeFinding { start, end, message: format!("{who} has a complexity of {count} (max {max})"), edits: Vec::new() });
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

// ---- max-depth (W-132) ----

/// Whether a statement opens a block that counts toward nesting. `else if`
/// continues its chain and adds none.
fn is_depth_block(n: Node<'_>, py: bool) -> bool {
    match n.kind() {
        "if_statement" => !n.parent().is_some_and(|p| p.kind() == "else_clause"),
        "for_statement" | "while_statement" | "try_statement" | "with_statement" => true,
        "switch_statement" | "for_in_statement" | "do_statement" => !py,
        "match_statement" => py,
        _ => false,
    }
}

fn depth_walk(n: Node<'_>, level: usize, max: usize, rule: &Size, src: &str, out: &mut Vec<NativeFinding>) {
    let mut cur = n.walk();
    let kids: Vec<Node<'_>> = n.children(&mut cur).collect();
    for c in kids {
        if is_function(c.kind(), rule.py) {
            continue;
        }
        let mut next = level;
        if is_depth_block(c, rule.py) {
            next += 1;
            if next > max {
                let (start, end) = function_span(c, None, src);
                out.push(NativeFinding { start, end, message: format!("Blocks are nested {next} deep (max {max})"), edits: Vec::new() });
            }
        }
        depth_walk(c, next, max, rule, src, out);
    }
}

fn depth(rule: &Size, root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let max = uint(ctx.options, "max", 4);
    // The file's own statements and every function start at depth 0.
    depth_walk(root, 0, max, rule, ctx.source, out);
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n.id() != root.id() && is_function(n.kind(), rule.py) {
            depth_walk(n, 0, max, rule, ctx.source, out);
        }
        if n.child_count() > 0 {
            let mut cur = n.walk();
            let kids: Vec<Node<'_>> = n.children(&mut cur).collect();
            stack.extend(kids.into_iter().rev());
        }
    }
}

// ---- React components (W-132) ----

/// A React component: a function whose name starts with a capital letter
/// and that holds JSX (also when wrapped in `memo` or `forwardRef`), or a
/// class that extends `Component` or `PureComponent`. The node, the node
/// that names it and the name.
struct Component<'t> {
    node: Node<'t>,
    name_node: Option<Node<'t>>,
    name: String,
}

fn holds_jsx(f: Node<'_>) -> bool {
    own_nodes(f, false).into_iter().any(|x| matches!(x.kind(), "jsx_element" | "jsx_self_closing_element" | "jsx_fragment"))
}

/// The name a function component goes by: its own, or the variable it is
/// assigned to, looking through `memo(…)` and `forwardRef(…)`.
fn component_name<'t>(f: Node<'t>, src: &str) -> Option<(Option<Node<'t>>, String)> {
    if let Some(n) = f.child_by_field_name("name") {
        return Some((Some(n), text(n, src).to_owned()));
    }
    let mut cur = f;
    while let Some(p) = cur.parent() {
        match p.kind() {
            "arguments" => {
                let call = p.parent()?;
                let callee = call.child_by_field_name("function").map_or("", |c| text(c, src));
                if !(callee.ends_with("memo") || callee.ends_with("forwardRef")) {
                    return None;
                }
                cur = call;
            }
            "parenthesized_expression" | "as_expression" | "satisfies_expression" => cur = p,
            "variable_declarator" => {
                let n = p.child_by_field_name("name").filter(|c| c.kind() == "identifier")?;
                return Some((Some(n), text(n, src).to_owned()));
            }
            _ => return None,
        }
    }
    None
}

fn components<'t>(root: Node<'t>, src: &str) -> Vec<Component<'t>> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        match n.kind() {
            "function_declaration" | "function_expression" | "arrow_function" => {
                if let Some((name_node, name)) = component_name(n, src).filter(|(_, s)| crate::react::is_component_name(s))
                    && holds_jsx(n)
                {
                    out.push(Component { node: n, name_node, name });
                }
            }
            "class_declaration" | "class" if crate::react::is_component_class(n, src) => {
                if let Some(name_node) = n.child_by_field_name("name") {
                    out.push(Component { node: n, name_node: Some(name_node), name: text(name_node, src).to_owned() });
                }
            }
            _ => {}
        }
        if n.child_count() > 0 {
            let mut cur = n.walk();
            let kids: Vec<Node<'_>> = n.children(&mut cur).collect();
            stack.extend(kids.into_iter().rev());
        }
    }
    out.sort_by_key(|c| c.node.start_byte());
    out
}

fn component_lines(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let o = ctx.options;
    let max = uint(o, "max", 200);
    let skips = Skips::read(o, false);
    let m = Measured::new(root, ctx.source, false, &skips);
    let note = skips.note();
    let note = if note.is_empty() { String::new() } else { format!("; {note}") };
    for c in components(root, ctx.source) {
        if name_matches(o, "ignore_names", &c.name) {
            continue;
        }
        let (first, last) = rows(c.node);
        let count = m.rows(first, last);
        if count > max {
            let (start, end) = function_span(c.node, c.name_node, ctx.source);
            out.push(NativeFinding { start, end, message: format!("Component `{}` is {count} lines (max {max}{note})", c.name), edits: Vec::new() });
        }
    }
}

/// Members of an object type or an interface body.
fn type_members(body: Node<'_>) -> usize {
    let mut cur = body.walk();
    body.named_children(&mut cur).filter(|c| matches!(c.kind(), "property_signature" | "method_signature")).count()
}

/// The members of the type named `name`, declared in this file.
fn declared_members(root: Node<'_>, name: &str, src: &str) -> usize {
    let mut best = 0;
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        match n.kind() {
            "interface_declaration" | "type_alias_declaration" if n.child_by_field_name("name").is_some_and(|x| text(x, src) == name) => {
                let body = n.child_by_field_name("body").or_else(|| n.child_by_field_name("value"));
                if let Some(b) = body.filter(|b| matches!(b.kind(), "object_type" | "interface_body")) {
                    best = best.max(type_members(b));
                }
            }
            "program" | "export_statement" => {
                let mut cur = n.walk();
                stack.extend(n.named_children(&mut cur).collect::<Vec<_>>());
            }
            _ => {}
        }
    }
    best
}

/// How many props a component's first parameter names: the larger of the
/// keys it destructures and the members of its declared type.
fn prop_count(f: Node<'_>, root: Node<'_>, src: &str) -> usize {
    let param = match f.child_by_field_name("parameters") {
        Some(ps) => {
            let mut cur = ps.walk();
            ps.named_children(&mut cur).find(|c| c.kind() != "comment")
        }
        None => f.child_by_field_name("parameter"),
    };
    let Some(param) = param else { return 0 };
    let pattern = if matches!(param.kind(), "required_parameter" | "optional_parameter") { param.child_by_field_name("pattern").unwrap_or(param) } else { param };
    let pattern = if pattern.kind() == "assignment_pattern" { pattern.child_by_field_name("left").unwrap_or(pattern) } else { pattern };
    let mut destructured = 0;
    if pattern.kind() == "object_pattern" {
        let mut cur = pattern.walk();
        destructured = pattern.named_children(&mut cur).filter(|c| matches!(c.kind(), "shorthand_property_identifier_pattern" | "pair_pattern" | "object_assignment_pattern")).count();
    }
    let mut declared = 0;
    if let Some(ty) = param.child_by_field_name("type") {
        let mut cur = ty.walk();
        if let Some(inner) = ty.named_children(&mut cur).next() {
            declared = match inner.kind() {
                "object_type" => type_members(inner),
                "type_identifier" => declared_members(root, text(inner, src), src),
                _ => 0,
            };
        }
    }
    destructured.max(declared)
}

fn component_props(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let o = ctx.options;
    let max = uint(o, "max", 8);
    for c in components(root, ctx.source) {
        if c.node.kind().starts_with("class") || name_matches(o, "ignore_names", &c.name) {
            continue;
        }
        let count = prop_count(c.node, root, ctx.source);
        if count > max {
            let (start, end) = function_span(c.node, c.name_node, ctx.source);
            let noun = if count == 1 { "prop" } else { "props" };
            out.push(NativeFinding { start, end, message: format!("Component `{}` takes {count} {noun} (max {max})", c.name), edits: Vec::new() });
        }
    }
}

fn is_jsx(kind: &str) -> bool {
    matches!(kind, "jsx_element" | "jsx_fragment" | "jsx_self_closing_element")
}

/// The deepest JSX nesting at or below `n`, counting `n` as `level`.
fn deepest_jsx(n: Node<'_>, level: usize) -> usize {
    let mut best = level;
    let mut cur = n.walk();
    for c in n.children(&mut cur) {
        let l = if is_jsx(c.kind()) { level + 1 } else { level };
        best = best.max(deepest_jsx(c, l));
    }
    best
}

fn jsx_depth(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let max = uint(ctx.options, "max", 6);
    // Nesting is counted through any `{…}` between elements. The first
    // element past `max` is reported, with how deep its own branch goes.
    let mut stack: Vec<(Node<'_>, usize)> = vec![(root, 0)];
    while let Some((n, level)) = stack.pop() {
        let mut here = level;
        if is_jsx(n.kind()) {
            here += 1;
            if here == max + 1 {
                let deepest = deepest_jsx(n, here);
                let (start, end) = function_span(n, None, ctx.source);
                out.push(NativeFinding { start, end, message: format!("JSX is nested {deepest} deep (max {max})"), edits: Vec::new() });
                continue;
            }
        }
        if n.child_count() > 0 {
            let mut cur = n.walk();
            let kids: Vec<Node<'_>> = n.children(&mut cur).collect();
            stack.extend(kids.into_iter().rev().map(|k| (k, here)));
        }
    }
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

    const JS_CX: Size = Size::js("js/complexity", Measure::Complexity);
    const REACT_LINES: Size = Size::react("react/component-max-lines", Measure::ComponentLines);
    const REACT_PROPS: Size = Size::react("react/max-props", Measure::Props);
    const REACT_JSX: Size = Size::react("react/max-jsx-depth", Measure::JsxDepth);
    const PY_CX: Size = Size::py("py/complexity", Measure::Complexity);
    const JS_DEPTH: Size = Size::js("js/max-depth", Measure::Depth);
    const PY_DEPTH: Size = Size::py("py/max-depth", Measure::Depth);

    #[test]
    fn js_complexity_counts_branches_not_nested_functions() {
        let src = "function f(a, b) {\n  if (a && b || a) { return 1; }\n  for (const x of a) { while (x) { x = a ?? b; } }\n  switch (a) { case 1: break; case 2: break; default: break; }\n  try { a(); } catch (e) { b(); }\n  const g = () => { if (a) { b(); } };\n  return a ? 1 : 2;\n}\n";
        // 1 + if + && + || + for + while + ?? + 2 cases + catch + ternary = 11
        let f = js(&JS_CX, src, json!({ "max": 10 }));
        assert_eq!(messages(&f), ["`f` has a complexity of 11 (max 10)"]);
        assert!(js(&JS_CX, src, json!({ "max": 11 })).is_empty());
        let f = js(&JS_CX, src, json!({ "max": 1, "ignore_names": "^f$" }));
        assert_eq!(messages(&f), ["`g` has a complexity of 2 (max 1)"]);
        let chain = "function f(a) { return a?.b?.c; }";
        assert!(js(&JS_CX, chain, json!({ "max": 1 })).is_empty());
        assert_eq!(messages(&js(&JS_CX, chain, json!({ "max": 1, "count_optional_chain": true }))), ["`f` has a complexity of 3 (max 1)"]);
        let assign = "function f(a) { a ||= 1; a &&= 2; a ??= 3; }";
        assert_eq!(messages(&js(&JS_CX, assign, json!({ "max": 1 }))), ["`f` has a complexity of 4 (max 1)"]);
    }

    #[test]
    fn python_complexity() {
        let src = "def f(a, b):\n    if a and b or a:\n        pass\n    elif b:\n        pass\n    for x in a:\n        while x:\n            pass\n    try:\n        pass\n    except E:\n        pass\n    y = [i for i in a if i]\n    z = 1 if a else 2\n    match a:\n        case 1:\n            pass\n        case _:\n            pass\n    def g():\n        if a:\n            pass\n";
        // 1 + if + and + or + elif + for + while + except + for_in + if_clause + ternary + case 1 = 12
        let f = py(&PY_CX, src, json!({ "max": 11 }));
        assert_eq!(messages(&f), ["`f` has a complexity of 12 (max 11)"]);
        assert!(py(&PY_CX, src, json!({ "max": 12 })).is_empty());
    }

    #[test]
    fn react_component_lines() {
        let src = "const Big = () => {\n  const a = 1;\n  const b = 2;\n  return <div>{a}{b}</div>;\n};\nfunction helper() {\n  const a = 1;\n  const b = 2;\n  const c = 3;\n  return a + b + c;\n}\nconst Wrapped = memo(forwardRef((p, r) => {\n  const a = 1;\n  const b = 2;\n  return <i ref={r} />;\n}));\nclass K extends React.Component {\n  render() {\n    const a = 1;\n    return <b />;\n  }\n}\n";
        let f = run(&REACT_LINES, Language::Jsx, "a.jsx", src, json!({ "max": 4 }));
        assert_eq!(messages(&f), ["Component `Big` is 5 lines (max 4; blank lines and comments not counted)", "Component `Wrapped` is 5 lines (max 4; blank lines and comments not counted)", "Component `K` is 6 lines (max 4; blank lines and comments not counted)"]);
        let f = run(&REACT_LINES, Language::Jsx, "a.jsx", src, json!({ "max": 4, "ignore_names": "^(Big|K)$" }));
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn react_props() {
        let src = "function A({ a, b, c = 1, d: e, ...rest }) { return <i />; }\ninterface BProps { a: string; b: number; onX(): void }\nconst B = (props: BProps) => <b />;\nconst C = ({ a }: { a: 1; b: 2; c: 3; d: 4 }) => <b />;\nfunction notAComponent({ a, b, c, d }) { return 1; }\n";
        let tsx = |o: Value| run(&REACT_PROPS, Language::Tsx, "a.tsx", src, o);
        assert_eq!(messages(&tsx(json!({ "max": 3 }))), ["Component `A` takes 4 props (max 3)", "Component `C` takes 4 props (max 3)"]);
        assert_eq!(messages(&tsx(json!({ "max": 2 }))), ["Component `A` takes 4 props (max 2)", "Component `B` takes 3 props (max 2)", "Component `C` takes 4 props (max 2)"]);
    }

    #[test]
    fn react_jsx_depth() {
        let src = "const A = () => (\n  <a>\n    <b>\n      {xs.map((x) => (\n        <c>\n          <d />\n          <e><f /></e>\n        </c>\n      ))}\n    </b>\n  </a>\n);\n";
        let f = run(&REACT_JSX, Language::Jsx, "a.jsx", src, json!({ "max": 2 }));
        assert_eq!(messages(&f), ["JSX is nested 5 deep (max 2)"]);
        assert_eq!(&src[f[0].start..f[0].end], "<c>");
        assert!(run(&REACT_JSX, Language::Jsx, "a.jsx", src, json!({ "max": 5 })).is_empty());
    }

    #[test]
    fn js_depth() {
        let src = "function f(a) {\n  if (a) {\n    for (;;) {\n      while (a) {\n        if (a) {\n          try { a(); } catch (e) {}\n        } else if (a) { a(); }\n      }\n    }\n  }\n}\n";
        let f = js(&JS_DEPTH, src, json!({ "max": 3 }));
        assert_eq!(messages(&f), ["Blocks are nested 4 deep (max 3)", "Blocks are nested 5 deep (max 3)"]);
        assert_eq!(&src[f[0].start..f[0].end], "if (a) {");
        assert!(js(&JS_DEPTH, src, json!({ "max": 5 })).is_empty());
        // a function resets the count
        let nested = "if (a) { if (b) { const f = () => { if (c) { if (d) {} } }; } }";
        assert!(js(&JS_DEPTH, nested, json!({ "max": 2 })).is_empty());
    }

    #[test]
    fn python_depth() {
        let src = "def f(a):\n    if a:\n        for x in a:\n            with x:\n                if x:\n                    pass\n                elif a:\n                    pass\n";
        let f = py(&PY_DEPTH, src, json!({ "max": 3 }));
        assert_eq!(messages(&f), ["Blocks are nested 4 deep (max 3)"]);
        assert!(py(&PY_DEPTH, src, json!({ "max": 4 })).is_empty());
    }

    #[test]
    fn data_carries_the_value_and_the_limit() {
        let f = NativeFinding { start: 0, end: 0, message: "`f` has a complexity of 24 (max 20)".into(), edits: Vec::new() };
        assert_eq!(JS_CX.data_of(&f, &BTreeMap::new()), Some(json!({ "complexity": 24, "max": 20 })));
        let f = NativeFinding { start: 0, end: 0, message: "Blocks are nested 5 deep (max 4)".into(), edits: Vec::new() };
        assert_eq!(JS_DEPTH.data_of(&f, &BTreeMap::new()), Some(json!({ "depth": 5, "max": 4 })));
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
