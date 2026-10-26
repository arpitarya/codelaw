//! The functions of a JS, TS or Python file, and what a change did to each
//! (W-71 step 1; SR-BASELINE decision 3's `on_touch_scope = "function"` needs
//! the same answer).
//!
//! - [`functions`] lists the functions a test rule judges: name (qualified,
//!   `Invoice.total`), byte and line span, whether the module exports it, and
//!   the token stream of its text. Not listed: anonymous callbacks, nested
//!   functions (judged through the function that holds them), overload
//!   signatures, abstract methods and `.d.ts`-style declarations.
//! - [`classify`] sets a file's functions against the same file at the base
//!   revision: **new** (no function of that name there), **edited** (its token
//!   stream differs, so reformatting, comment edits and moves do not count) or
//!   **untouched**. No base copy: a new file, every function is new.
//!
//! Same-named functions in one file are told apart by position: `f#1`, `f#2`.

use crate::{Token, parse};
use codelaw_core::Language;
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::Node;

/// One function of a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    /// The qualified name; `f#1`, `f#2` when the file has several of one name.
    pub name: String,
    pub start_byte: usize,
    pub end_byte: usize,
    /// 1-based, inclusive.
    pub first_line: u32,
    pub last_line: u32,
    /// Exported (JS/TS); no leading `_`, or in `__all__` (Python).
    pub public: bool,
    /// The function's token stream: depths are relative to the function.
    pub tokens: Vec<Token>,
}

/// What a change did to a function.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Change {
    New,
    Edited,
    Untouched,
}

impl Change {
    pub fn name(self) -> &'static str {
        match self {
            Change::New => "new",
            Change::Edited => "edited",
            Change::Untouched => "untouched",
        }
    }
}

/// Whether [`functions`] reads `language`.
pub fn has_functions(language: Language) -> bool {
    matches!(language, Language::Javascript | Language::Jsx | Language::Typescript | Language::Tsx | Language::Python)
}

/// The functions of `source` parsed as `language`, in source order.
pub fn functions(language: Language, source: &str) -> Vec<Function> {
    if !has_functions(language) {
        return Vec::new();
    }
    let root = parse(language, source);
    let r = root.root();
    let top = r.get_inner_node();
    let mut w = Walker { src: source, out: Vec::new(), exports: BTreeSet::new(), all: None };
    if language == Language::Python {
        w.all = python_all(top, source);
        w.py(top, "", true);
    } else {
        w.exports = js_exports(top, source);
        w.js(top, &Scope { prefix: String::new(), exported: false });
    }
    number_duplicates(w.out)
}

/// Each of `current`'s functions against the file at the base revision
/// (`None`: the file has no copy there).
pub fn classify(current: &[Function], base: Option<&[Function]>) -> Vec<Change> {
    let Some(base) = base else { return vec![Change::New; current.len()] };
    let before: BTreeMap<&str, &Function> = base.iter().map(|f| (f.name.as_str(), f)).collect();
    current
        .iter()
        .map(|f| match before.get(f.name.as_str()) {
            None => Change::New,
            Some(b) if b.tokens == f.tokens => Change::Untouched,
            Some(_) => Change::Edited,
        })
        .collect()
}

fn number_duplicates(mut fns: Vec<Function>) -> Vec<Function> {
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for f in &fns {
        *count.entry(f.name.clone()).or_default() += 1;
    }
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for f in &mut fns {
        if count[&f.name] > 1 {
            let n = seen.entry(f.name.clone()).or_default();
            *n += 1;
            f.name = format!("{}#{n}", f.name);
        }
    }
    fns
}

#[derive(Clone)]
struct Scope {
    prefix: String,
    exported: bool,
}

struct Walker<'s> {
    src: &'s str,
    out: Vec<Function>,
    /// JS: names listed in `export { a, b }`.
    exports: BTreeSet<String>,
    /// Python: `__all__`, when the module sets one.
    all: Option<BTreeSet<String>>,
}

fn text<'s>(n: Node<'_>, src: &'s str) -> &'s str {
    &src[n.byte_range()]
}

fn named<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    let mut c = n.walk();
    n.named_children(&mut c).collect()
}

fn qualify(prefix: &str, name: &str) -> String {
    if prefix.is_empty() { name.to_owned() } else { format!("{prefix}.{name}") }
}

/// A node's tokens, depths relative to it: every node in pre-order, each leaf
/// with its text. Comments are nodes, so they count; whitespace does not.
fn tokens_of(n: Node<'_>, src: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut cursor = n.walk();
    let mut depth = 0u32;
    loop {
        let node = cursor.node();
        let leaf = node.child_count() == 0;
        if !(leaf && node.start_byte() == node.end_byte()) {
            out.push(Token { depth, kind: node.kind().to_owned(), text: if leaf { src[node.byte_range()].to_owned() } else { String::new() } });
        }
        if cursor.goto_first_child() {
            depth += 1;
            continue;
        }
        loop {
            if cursor.node().id() == n.id() {
                return out;
            }
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return out;
            }
            depth -= 1;
        }
    }
}

impl Walker<'_> {
    fn push(&mut self, span: Node<'_>, name: String, public: bool) {
        let (s, e) = (span.start_position(), span.end_position());
        // A node ending at column 0 does not own that row.
        let last_row = if e.column == 0 && e.row > s.row { e.row - 1 } else { e.row };
        self.out.push(Function {
            name,
            start_byte: span.start_byte(),
            end_byte: span.end_byte(),
            first_line: s.row as u32 + 1,
            last_line: last_row as u32 + 1,
            public,
            tokens: tokens_of(span, self.src),
        });
    }

    // ---- JavaScript and TypeScript ----

    fn js(&mut self, n: Node<'_>, sc: &Scope) {
        match n.kind() {
            "program" | "class_body" => {
                for c in named(n) {
                    self.js(c, sc);
                }
            }
            "internal_module" | "module" => {
                let prefix = n.child_by_field_name("name").map_or(String::new(), |c| qualify(&sc.prefix, text(c, self.src)));
                if let Some(body) = n.child_by_field_name("body") {
                    for c in named(body) {
                        self.js(c, &Scope { prefix: prefix.clone(), exported: sc.exported });
                    }
                }
            }
            "export_statement" => {
                let inner = Scope { exported: true, ..sc.clone() };
                for c in named(n) {
                    if is_js_fn(unwrap(c)) && c.kind() != "function_declaration" && c.kind() != "generator_function_declaration" {
                        let f = unwrap(c);
                        self.push(f, qualify(&sc.prefix, f.child_by_field_name("name").map_or("default", |x| text(x, self.src))), true);
                    } else {
                        self.js(c, &inner);
                    }
                }
            }
            "function_declaration" | "generator_function_declaration" => {
                if let Some(name) = n.child_by_field_name("name") {
                    let nm = text(name, self.src);
                    let public = sc.exported || (sc.prefix.is_empty() && self.exports.contains(nm));
                    self.push(n, qualify(&sc.prefix, nm), public);
                }
            }
            "class_declaration" | "class" | "abstract_class_declaration" => {
                let nm = n.child_by_field_name("name").map_or("default", |c| text(c, self.src));
                self.js_class(n, nm, sc);
            }
            "method_definition" => {
                if let Some(name) = n.child_by_field_name("name") {
                    let private = name.kind() == "private_property_identifier" || has_modifier(n, self.src, &["private", "protected"]);
                    self.push(n, qualify(&sc.prefix, text(name, self.src)), sc.exported && !private);
                }
            }
            "field_definition" | "public_field_definition" => {
                let name = n.child_by_field_name("property").or_else(|| n.child_by_field_name("name"));
                if let (Some(name), Some(v)) = (name, n.child_by_field_name("value"))
                    && is_js_fn(unwrap(v))
                {
                    let private = name.kind() == "private_property_identifier" || has_modifier(n, self.src, &["private", "protected"]);
                    self.push(n, qualify(&sc.prefix, text(name, self.src)), sc.exported && !private);
                }
            }
            "lexical_declaration" | "variable_declaration" => {
                let decls: Vec<Node<'_>> = named(n).into_iter().filter(|c| c.kind() == "variable_declarator").collect();
                let single = decls.len() == 1;
                for d in decls {
                    let (Some(name), Some(v)) = (d.child_by_field_name("name"), d.child_by_field_name("value")) else { continue };
                    if name.kind() != "identifier" {
                        continue;
                    }
                    let nm = text(name, self.src).to_owned();
                    let public = sc.exported || (sc.prefix.is_empty() && self.exports.contains(&nm));
                    let value = unwrap(v);
                    if is_js_fn(value) {
                        self.push(if single { n } else { d }, qualify(&sc.prefix, &nm), public);
                    } else if matches!(value.kind(), "class" | "class_declaration") {
                        self.js_class(value, &nm, &Scope { prefix: sc.prefix.clone(), exported: public });
                    } else if value.kind() == "object" {
                        self.js_object(value, &Scope { prefix: qualify(&sc.prefix, &nm), exported: public });
                    }
                }
            }
            "expression_statement" => {
                for c in named(n) {
                    if c.kind() != "assignment_expression" {
                        continue;
                    }
                    let (Some(l), Some(r)) = (c.child_by_field_name("left"), c.child_by_field_name("right")) else { continue };
                    let lt = text(l, self.src);
                    let public = lt == "exports" || lt.starts_with("exports.") || lt.starts_with("module.exports");
                    let value = unwrap(r);
                    if is_js_fn(value) && matches!(l.kind(), "identifier" | "member_expression") {
                        self.push(n, lt.to_owned(), public);
                    } else if value.kind() == "object" && lt == "module.exports" {
                        self.js_object(value, &Scope { prefix: String::new(), exported: true });
                    }
                }
            }
            _ => {}
        }
    }

    fn js_class(&mut self, class: Node<'_>, name: &str, sc: &Scope) {
        if let Some(body) = class.child_by_field_name("body") {
            self.js(body, &Scope { prefix: qualify(&sc.prefix, name), exported: sc.exported || (sc.prefix.is_empty() && self.exports.contains(name)) });
        }
    }

    fn js_object(&mut self, obj: Node<'_>, sc: &Scope) {
        for c in named(obj) {
            match c.kind() {
                "method_definition" => self.js(c, sc),
                "pair" => {
                    let (Some(k), Some(v)) = (c.child_by_field_name("key"), c.child_by_field_name("value")) else { continue };
                    let key = text(k, self.src).trim_matches(|ch| ch == '"' || ch == '\'');
                    let value = unwrap(v);
                    if is_js_fn(value) {
                        self.push(c, qualify(&sc.prefix, key), sc.exported);
                    } else if value.kind() == "object" {
                        self.js_object(value, &Scope { prefix: qualify(&sc.prefix, key), exported: sc.exported });
                    }
                }
                _ => {}
            }
        }
    }

    // ---- Python ----

    fn py(&mut self, n: Node<'_>, prefix: &str, public: bool) {
        for c in named(n) {
            match c.kind() {
                "function_definition" => self.py_def(c, c, prefix, public),
                "decorated_definition" => {
                    let Some(def) = c.child_by_field_name("definition") else { continue };
                    match def.kind() {
                        "function_definition" => {
                            let skip = named(c).iter().filter(|d| d.kind() == "decorator").any(|d| {
                                let t = text(*d, self.src);
                                let last = t.trim_start_matches('@').trim().split('(').next().unwrap_or("").rsplit('.').next().unwrap_or("");
                                matches!(last, "overload" | "abstractmethod")
                            });
                            if !skip {
                                self.py_def(def, c, prefix, public);
                            }
                        }
                        "class_definition" => self.py_class(def, prefix, public),
                        _ => {}
                    }
                }
                "class_definition" => self.py_class(c, prefix, public),
                "if_statement" | "elif_clause" | "else_clause" | "try_statement" | "except_clause" | "finally_clause" | "with_statement" | "for_statement" | "while_statement" | "block" => {
                    self.py(c, prefix, public)
                }
                _ => {}
            }
        }
    }

    fn py_def(&mut self, def: Node<'_>, span: Node<'_>, prefix: &str, public: bool) {
        let Some(name) = def.child_by_field_name("name") else { return };
        let nm = text(name, self.src);
        let top = prefix.is_empty();
        let listed = match (&self.all, top) {
            (Some(all), true) => all.contains(nm),
            _ => !nm.starts_with('_'),
        };
        self.push(span, qualify(prefix, nm), public && listed);
    }

    fn py_class(&mut self, class: Node<'_>, prefix: &str, public: bool) {
        let (Some(name), Some(body)) = (class.child_by_field_name("name"), class.child_by_field_name("body")) else { return };
        let nm = text(name, self.src);
        let listed = match (&self.all, prefix.is_empty()) {
            (Some(all), true) => all.contains(nm),
            _ => !nm.starts_with('_'),
        };
        self.py(body, &qualify(prefix, nm), public && listed);
    }
}

fn is_js_fn(n: Node<'_>) -> bool {
    matches!(n.kind(), "function_expression" | "function" | "generator_function" | "arrow_function" | "function_declaration" | "generator_function_declaration")
}

/// Look through parentheses and TS assertions.
fn unwrap(mut n: Node<'_>) -> Node<'_> {
    while matches!(n.kind(), "parenthesized_expression" | "as_expression" | "satisfies_expression" | "non_null_expression") {
        match named(n).first() {
            Some(c) => n = *c,
            None => break,
        }
    }
    n
}

fn has_modifier(n: Node<'_>, src: &str, words: &[&str]) -> bool {
    let mut c = n.walk();
    n.children(&mut c).any(|k| k.kind() == "accessibility_modifier" && words.contains(&text(k, src).trim()))
}

/// The names `export { a, b as c }` lists (the local names).
fn js_exports(top: Node<'_>, src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for s in named(top).into_iter().filter(|s| s.kind() == "export_statement") {
        if s.child_by_field_name("source").is_some() {
            continue;
        }
        for clause in named(s).into_iter().filter(|c| c.kind() == "export_clause") {
            for spec in named(clause) {
                if let Some(name) = spec.child_by_field_name("name") {
                    out.insert(text(name, src).to_owned());
                }
            }
        }
    }
    out
}

/// `__all__ = ["a", "b"]` (or `+=`) at module level.
fn python_all(top: Node<'_>, src: &str) -> Option<BTreeSet<String>> {
    let mut found: Option<BTreeSet<String>> = None;
    for s in named(top).into_iter().filter(|s| s.kind() == "expression_statement") {
        for a in named(s).into_iter().filter(|a| matches!(a.kind(), "assignment" | "augmented_assignment")) {
            let (Some(l), Some(r)) = (a.child_by_field_name("left"), a.child_by_field_name("right")) else { continue };
            if text(l, src) != "__all__" {
                continue;
            }
            let set = found.get_or_insert_with(BTreeSet::new);
            for item in named(r).into_iter().filter(|i| i.kind() == "string") {
                set.insert(text(item, src).trim_matches(|c| c == '"' || c == '\'').to_owned());
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(l: Language, src: &str) -> Vec<String> {
        functions(l, src).into_iter().map(|f| f.name).collect()
    }

    #[test]
    fn js_every_form_and_none_of_the_excluded() {
        let src = r#"
export function a() {}
const b = () => 1;
let c = function () {};
class K { m() {} get g() { return 1 } static s() {} f = () => 1; #p() {} }
const o = { x() {}, y: () => 1, z: { w: function () {} } };
items.map(x => x);
function outer() { function inner() {} const arrow = () => 1; }
exports.h = function () {};
export default () => 1;
"#;
        assert_eq!(names(Language::Javascript, src), ["a", "b", "c", "K.m", "K.g", "K.s", "K.f", "K.#p", "o.x", "o.y", "o.z.w", "outer", "exports.h", "default"]);
    }

    #[test]
    fn ts_overloads_abstract_and_declarations_are_not_functions() {
        let src = "function f(a: string): void;\nfunction f(a: number): void;\nfunction f(a: any) {}\nabstract class A { abstract m(): void; n(): void {} }\ndeclare function d(): void;\ninterface I { m(): void }\nconst g = (x: number) => x as number;\n";
        assert_eq!(names(Language::Typescript, src), ["f", "A.n", "g"]);
    }

    #[test]
    fn ts_publicness() {
        let src = "export class A { m() {} private p() {} protected q() {} #r() {} }\nclass B { m() {} }\nfunction h() {}\nexport { h };\nexport const k = () => 1;\n";
        let f = functions(Language::Typescript, src);
        let pub_: Vec<(&str, bool)> = f.iter().map(|f| (f.name.as_str(), f.public)).collect();
        assert_eq!(pub_, [("A.m", true), ("A.p", false), ("A.q", false), ("A.#r", false), ("B.m", false), ("h", true), ("k", true)]);
    }

    #[test]
    fn tsx_and_jsx_components() {
        assert_eq!(names(Language::Tsx, "export const C = (p: P) => <div>{p.a}</div>;\n"), ["C"]);
        assert_eq!(names(Language::Jsx, "function C() { return <a/>; }\n"), ["C"]);
    }

    #[test]
    fn python_module_class_async_decorators() {
        let src = "def a(): pass\nasync def b(): pass\nclass C:\n    def m(self): pass\n    @property\n    def p(self): pass\n    @overload\n    def o(self): ...\n    def _hid(self): pass\n    def n(self):\n        def inner(): pass\nif X:\n    def cond(): pass\nf = lambda: 1\n";
        assert_eq!(names(Language::Python, src), ["a", "b", "C.m", "C.p", "C._hid", "C.n", "cond"]);
    }

    #[test]
    fn python_publicness_and_all() {
        let f = functions(Language::Python, "def a(): pass\ndef _b(): pass\nclass C:\n    def m(self): pass\n    def _n(self): pass\n");
        assert_eq!(f.iter().map(|f| f.public).collect::<Vec<_>>(), [true, false, true, false]);
        let f = functions(Language::Python, "__all__ = [\"b\"]\ndef a(): pass\ndef b(): pass\n");
        assert_eq!(f.iter().map(|f| f.public).collect::<Vec<_>>(), [false, true]);
    }

    #[test]
    fn same_name_is_numbered_by_position() {
        let f = functions(Language::Python, "if A:\n    def f(): pass\nelse:\n    def f(): pass\n");
        assert_eq!(f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["f#1", "f#2"]);
    }

    #[test]
    fn spans_are_one_based_and_inclusive() {
        let f = functions(Language::Python, "x = 1\n\ndef a():\n    return 1\n\ndef b(): pass\n");
        assert_eq!((f[0].first_line, f[0].last_line), (3, 4));
        assert_eq!((f[1].first_line, f[1].last_line), (6, 6));
    }

    fn classify_js(base: &str, now: &str) -> Vec<(String, Change)> {
        let b = functions(Language::Typescript, base);
        let n = functions(Language::Typescript, now);
        let c = classify(&n, Some(&b));
        n.into_iter().map(|f| f.name).zip(c).collect()
    }

    #[test]
    fn a_reformat_a_move_and_a_comment_outside_edit_nothing() {
        let base = "function a(x: number) { return x + 1; }\nfunction b() { return 2; }\n";
        let now = "// header\nfunction b() {\n  return 2;\n}\n\nfunction a(x: number) {\n  return x + 1;\n}\n";
        assert_eq!(classify_js(base, now), [("b".into(), Change::Untouched), ("a".into(), Change::Untouched)]);
    }

    #[test]
    fn a_changed_token_or_inner_comment_is_an_edit_and_a_rename_is_new() {
        let base = "function a() { return 1; }\nfunction b() { return 2; }\nfunction c() { return 3; }\n";
        let now = "function a() { return 9; }\nfunction b() { /* why */ return 2; }\nfunction renamed() { return 3; }\n";
        assert_eq!(classify_js(base, now), [("a".into(), Change::Edited), ("b".into(), Change::Edited), ("renamed".into(), Change::New)]);
    }

    #[test]
    fn python_indentation_depth_is_part_of_the_tokens() {
        let b = functions(Language::Python, "def a():\n    if x:\n        y()\n    z()\n");
        let n = functions(Language::Python, "def a():\n    if x:\n        y()\n        z()\n");
        assert_eq!(classify(&n, Some(&b)), [Change::Edited]);
    }

    #[test]
    fn no_base_copy_is_a_new_file() {
        let n = functions(Language::Python, "def a(): pass\ndef b(): pass\n");
        assert_eq!(classify(&n, None), [Change::New, Change::New]);
    }

    #[test]
    fn other_languages_have_no_functions() {
        assert!(functions(Language::Json, "{}").is_empty());
    }
}
