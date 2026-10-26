//! CSS Modules facts (W-63, SR-PROJECT-GRAPH decision 8): what a stylesheet
//! defines as a local class, what it composes, and what it exports to
//! JavaScript (`:export`, `@value`).
//!
//! Read from the text, not from a syntax tree: tree-sitter-scss cannot read all
//! modern Sass, and a class the grammar missed would be reported as undefined.
//! The scan knows strings, comments, `url()`, `#{}` interpolation, `:global`,
//! `:local` and Sass's `&-suffix`; when it meets what it cannot name (a
//! suffix on a parent that is not one plain class, an unbalanced block) it
//! marks the module `incomplete`, and the rules that need every class stay
//! silent for it.

use crate::facts::Span;
use serde::{Deserialize, Serialize};

/// A local class a module defines.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModClass {
    pub name: String,
    /// The name, without its dot.
    pub span: Span,
    /// The whole rule, extended to whole lines, when this class is the only
    /// selector of a rule with no nested rule: deleting it removes the class.
    pub rule: Option<Span>,
}

/// Where a `composes:` takes its classes from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComposeFrom {
    /// The same file.
    Local,
    /// `from global`.
    Global,
    /// `from "./other.module.css"`.
    File(String),
}

/// One `composes: a b from …` declaration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compose {
    /// The declaration, from `composes` to its end.
    pub span: Span,
    pub names: Vec<(String, Span)>,
    pub from: ComposeFrom,
    /// The rule's selector is not one top-level class selector, which
    /// `composes` needs.
    pub misplaced: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleFacts {
    pub classes: Vec<ModClass>,
    pub composes: Vec<Compose>,
    /// `:export { name: … }` keys and `@value name: …` names: what JavaScript
    /// can read besides the classes.
    pub exports: Vec<String>,
    /// A class name could not be worked out (a `&-suffix` on a parent that is
    /// not one plain class, an interpolated selector, an unbalanced block).
    pub incomplete: bool,
}

/// Whether a path names a CSS Module by its file name.
pub fn named_module(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".module.css") || lower.ends_with(".module.scss")
}

/// The names JavaScript reaches a class by, under css-loader's
/// `localsConvention` (`[css] modules_convention`).
pub fn js_names(class: &str, convention: &str) -> Vec<String> {
    let camel = |dashes_only: bool| -> String {
        let mut out = String::new();
        let mut up = false;
        for c in class.chars() {
            if c == '-' || (!dashes_only && c == '_') {
                up = true;
            } else if up {
                out.extend(c.to_uppercase());
                up = false;
            } else {
                out.push(c);
            }
        }
        out
    };
    match convention {
        "camelCase" => vec![class.to_owned(), camel(false)],
        "camelCaseOnly" => vec![camel(false)],
        "dashes" => vec![class.to_owned(), camel(true)],
        "dashesOnly" => vec![camel(true)],
        _ => vec![class.to_owned()],
    }
}

/// Scan a stylesheet. `scss` reads `//` as a comment.
pub fn scan(text: &str, scss: bool) -> ModuleFacts {
    let masked = mask(text, scss);
    let mut s = Scanner { text, m: masked.as_bytes(), out: ModuleFacts::default(), stack: Vec::new(), interp: false };
    s.run();
    s.out.classes.sort_by_key(|c| c.span);
    s.out.exports.sort();
    s.out.exports.dedup();
    s.out
}

/// `text` with comments, string contents and unquoted `url(…)` blanked
/// (same length, newlines kept).
fn mask(text: &str, scss: bool) -> String {
    let b = text.as_bytes();
    let mut out = b.to_vec();
    let blank = |out: &mut Vec<u8>, from: usize, to: usize| {
        for x in &mut out[from..to] {
            if *x != b'\n' {
                *x = b' ';
            }
        }
    };
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => {
                let q = b[i];
                let mut j = i + 1;
                while j < b.len() && b[j] != q {
                    j += if b[j] == b'\\' { 2 } else { 1 };
                }
                let end = j.min(b.len());
                blank(&mut out, i + 1, end);
                i = (end + 1).min(b.len());
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let end = text[i + 2..].find("*/").map_or(b.len(), |o| i + 2 + o + 2);
                blank(&mut out, i, end);
                i = end;
            }
            b'/' if scss && b.get(i + 1) == Some(&b'/') => {
                let end = text[i..].find('\n').map_or(b.len(), |o| i + o);
                blank(&mut out, i, end);
                i = end;
            }
            b'u' | b'U' if text[i..].len() >= 4 && text[i..i + 4].eq_ignore_ascii_case("url(") && (i == 0 || !is_ident(b[i - 1])) => {
                if text[i + 4..].trim_start().starts_with(['"', '\'']) {
                    i += 4;
                } else {
                    let end = text[i..].find(')').map_or(b.len(), |o| i + o);
                    blank(&mut out, i + 4, end);
                    i = end;
                }
            }
            _ => i += 1,
        }
    }
    String::from_utf8(out).expect("blanking ASCII-delimited runs keeps UTF-8")
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b >= 0x80
}

/// One open block.
struct Block {
    /// What opened it.
    kind: Kind,
    /// The one plain class its selector names, for `&-suffix` and for `composes`.
    class: Option<String>,
    /// Where its prelude started, and how many classes it defined (their
    /// indices in `out.classes`), so a lone-class rule can be recorded.
    start: usize,
    defined: Vec<usize>,
    lone: bool,
    /// Rules nested inside.
    nested: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Rule,
    /// A conditional or grouping at-rule (`@media`, `@supports`, `@layer`,
    /// `@include x { }`, `@if`), whose rules are the stylesheet's own.
    Group,
    /// `@keyframes` and the like: no classes inside.
    Opaque,
    /// `:export { … }`.
    Export,
}

struct Scanner<'a> {
    text: &'a str,
    /// `text` masked (comments, strings, URLs blanked), as bytes.
    m: &'a [u8],
    out: ModuleFacts,
    stack: Vec<Block>,
    /// The statement being read holds a `#{}` interpolation.
    interp: bool,
}

impl Scanner<'_> {
    fn run(&mut self) {
        let n = self.m.len();
        let mut stmt = 0;
        let mut i = 0;
        let mut depth_paren = 0usize;
        while i < n {
            match self.m[i] {
                b'#' if self.m.get(i + 1) == Some(&b'{') => {
                    // Interpolation: skip to its matching brace.
                    let mut d = 0;
                    while i < n {
                        match self.m[i] {
                            b'{' => d += 1,
                            b'}' => {
                                d -= 1;
                                if d == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    i += 1;
                    self.interp = true;
                }
                b'(' | b'[' => {
                    depth_paren += 1;
                    i += 1;
                }
                b')' | b']' => {
                    depth_paren = depth_paren.saturating_sub(1);
                    i += 1;
                }
                b'{' if depth_paren == 0 => {
                    self.open(stmt, i);
                    self.interp = false;
                    i += 1;
                    stmt = i;
                }
                b';' if depth_paren == 0 => {
                    self.statement(stmt, i);
                    self.interp = false;
                    i += 1;
                    stmt = i;
                }
                b'}' if depth_paren == 0 => {
                    self.statement(stmt, i);
                    self.close(i);
                    self.interp = false;
                    i += 1;
                    stmt = i;
                }
                _ => i += 1,
            }
        }
        if !self.stack.is_empty() {
            self.out.incomplete = true;
        }
    }

    fn within_opaque(&self) -> bool {
        self.stack.iter().any(|b| b.kind == Kind::Opaque)
    }

    /// A block opens: its prelude is `m[stmt..at]`.
    fn open(&mut self, stmt: usize, at: usize) {
        let masked = std::str::from_utf8(&self.m[stmt..at]).unwrap_or_default();
        let prelude = masked.trim();
        let lead = masked.len() - masked.trim_start().len();
        let base = stmt + lead;
        if self.interp && !prelude.starts_with('@') {
            self.out.incomplete = true;
        }
        if let Some(rest) = prelude.strip_prefix('@') {
            let name: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-').collect::<String>().to_ascii_lowercase();
            let kind = if name.ends_with("keyframes") || matches!(name.as_str(), "font-face" | "property" | "page" | "counter-style" | "font-feature-values" | "mixin" | "function") {
                Kind::Opaque
            } else {
                Kind::Group
            };
            self.stack.push(Block { kind, class: None, start: base, defined: Vec::new(), lone: false, nested: false });
            return;
        }
        if prelude == ":export" {
            self.stack.push(Block { kind: Kind::Export, class: None, start: base, defined: Vec::new(), lone: false, nested: false });
            return;
        }
        if self.within_opaque() {
            self.stack.push(Block { kind: Kind::Opaque, class: None, start: base, defined: Vec::new(), lone: false, nested: false });
            return;
        }
        // A Sass nested property (`font: {`) is no selector.
        if prelude.ends_with(':') || prelude.split_once(": ").is_some_and(|(n, _)| n.bytes().all(is_ident) && !n.is_empty()) && !prelude.starts_with(['.', '&', '#', '%', ':', '[', '*']) {
            self.stack.push(Block { kind: Kind::Opaque, class: None, start: base, defined: Vec::new(), lone: false, nested: false });
            return;
        }
        let parent_class = self.stack.iter().rev().find(|b| b.kind == Kind::Rule).map(|b| b.class.clone());
        let mut defined = Vec::new();
        let mut class = None;
        let mut classes_found = 0usize;
        let mut commas = 0usize;
        let b = prelude.as_bytes();
        let mut j = 0;
        let mut global = false;
        // Parentheses opened by `:global(`, whose classes are global.
        let mut global_until: Option<usize> = None;
        let mut paren = 0usize;
        while j < b.len() {
            let c = b[j];
            match c {
                b',' if paren == 0 => {
                    commas += 1;
                    global = false;
                    j += 1;
                }
                b'(' => {
                    paren += 1;
                    j += 1;
                }
                b')' => {
                    paren = paren.saturating_sub(1);
                    if global_until == Some(paren) {
                        global_until = None;
                    }
                    j += 1;
                }
                b':' => {
                    let start = j + 1;
                    let mut e = start;
                    while e < b.len() && (is_ident(b[e]) || b[e] == b':') {
                        e += 1;
                    }
                    let name = prelude[start..e].trim_start_matches(':').to_ascii_lowercase();
                    match name.as_str() {
                        "global" if b.get(e) == Some(&b'(') => {
                            global_until = Some(paren);
                            j = e;
                        }
                        "global" => {
                            global = true;
                            j = e;
                        }
                        "local" if b.get(e) == Some(&b'(') => {
                            j = e;
                        }
                        "local" => {
                            global = false;
                            j = e;
                        }
                        _ => j = e.max(j + 1),
                    }
                }
                b'.' if j + 1 < b.len() && (is_ident(b[j + 1]) || b[j + 1] == b'\\') => {
                    let start = j + 1;
                    let mut e = start;
                    while e < b.len() && (is_ident(b[e]) || b[e] == b'\\') {
                        e += if b[e] == b'\\' { 2.min(b.len() - e) } else { 1 };
                    }
                    let raw = &prelude[start..e];
                    let name: String = raw.chars().filter(|c| *c != '\\').collect();
                    let is_global = global || global_until.is_some();
                    if !is_global && !self.within_opaque() {
                        let abs = base + start;
                        self.out.classes.push(ModClass { name: name.clone(), span: (abs, abs + raw.len()), rule: None });
                        defined.push(self.out.classes.len() - 1);
                        class = Some(name);
                    }
                    classes_found += 1;
                    j = e;
                }
                b'&' => {
                    // `&-suffix`: a class named from the parent's.
                    let mut e = j + 1;
                    while e < b.len() && is_ident(b[e]) {
                        e += 1;
                    }
                    if e > j + 1 {
                        let suffix = &prelude[j + 1..e];
                        match parent_class.clone().flatten() {
                            Some(p) => {
                                let name = format!("{p}{suffix}");
                                let abs = base + j;
                                self.out.classes.push(ModClass { name: name.clone(), span: (abs, abs + 1 + suffix.len()), rule: None });
                                defined.push(self.out.classes.len() - 1);
                                class = Some(name);
                                classes_found += 1;
                            }
                            None => self.out.incomplete = true,
                        }
                    }
                    j = e.max(j + 1);
                }
                b'#' if b.get(j + 1) == Some(&b'{') => {
                    self.out.incomplete = true;
                    j += 2;
                }
                _ => j += 1,
            }
        }
        let lone = commas == 0 && classes_found == 1 && defined.len() == 1 && prelude.starts_with('.') && prelude[1..].bytes().all(|c| is_ident(c) || c == b'\\');
        if commas > 0 || classes_found != 1 {
            class = None;
        }
        // A selector list's rule can't be the parent of a `&-suffix`.
        self.stack.push(Block { kind: Kind::Rule, class, start: base, defined, lone, nested: false });
        if let Some(parent) = self.stack.iter_mut().rev().nth(1) {
            parent.nested |= parent.kind == Kind::Rule;
        }
    }

    /// A block closes at `at` (its `}`).
    fn close(&mut self, at: usize) {
        let Some(b) = self.stack.pop() else { return };
        if b.kind == Kind::Rule && b.lone && !b.nested && b.defined.len() == 1 {
            let (s, e) = (b.start, at + 1);
            let ls = self.text[..s].rfind('\n').map_or(0, |i| i + 1);
            let start = if self.text[ls..s].trim().is_empty() { ls } else { s };
            let le = self.text[e..].find('\n').map_or(self.text.len(), |i| e + i + 1);
            let end = if self.text[e..le].trim().is_empty() { le } else { e };
            self.out.classes[b.defined[0]].rule = Some((start, end));
        }
    }

    /// A statement `m[from..to]`: a declaration, `@value`, `:export` entry.
    fn statement(&mut self, from: usize, to: usize) {
        if to <= from {
            return;
        }
        let raw = &self.text[from..to];
        let lead = raw.len() - raw.trim_start().len();
        let stmt = raw.trim();
        if stmt.is_empty() || self.m[from..to].iter().all(|b| b.is_ascii_whitespace()) {
            return;
        }
        let at = from + lead;
        if let Some(rest) = stmt.strip_prefix("@value") {
            if rest.starts_with(char::is_whitespace) {
                self.value(rest.trim());
            }
            return;
        }
        match self.stack.last().map(|b| b.kind) {
            Some(Kind::Export) => {
                if let Some((k, _)) = stmt.split_once(':') {
                    self.out.exports.push(k.trim().to_owned());
                }
            }
            Some(Kind::Rule) if stmt.split_once(':').is_some_and(|(k, _)| k.trim().eq_ignore_ascii_case("composes")) => {
                self.compose(at, stmt);
            }
            _ => {}
        }
    }

    /// `@value a: 1; ` / `@value a, b from "./f.css"`.
    fn value(&mut self, rest: &str) {
        if let Some((names, _)) = rest.split_once(" from ") {
            self.out.exports.extend(names.split(',').map(|n| n.split(" as ").last().unwrap_or(n).trim().to_owned()).filter(|n| !n.is_empty()));
        } else if let Some((name, _)) = rest.split_once(':') {
            let name = name.trim();
            if !name.is_empty() {
                self.out.exports.push(name.to_owned());
            }
        }
    }

    fn compose(&mut self, at: usize, stmt: &str) {
        let colon = stmt.find(':').expect("checked");
        let value = &stmt[colon + 1..];
        let value_at = at + colon + 1;
        let (names_part, from) = match value.rfind(" from ") {
            Some(i) => {
                let target = value[i + 6..].trim();
                let from = if target == "global" {
                    ComposeFrom::Global
                } else {
                    ComposeFrom::File(target.trim_matches(['"', '\'']).to_owned())
                };
                (&value[..i], from)
            }
            None => (value, ComposeFrom::Local),
        };
        // The file name was masked in the scan text but not in `self.text`;
        // `stmt` comes from `self.text`, so it is intact.
        let mut names = Vec::new();
        let mut off = 0;
        for w in names_part.split_inclusive(char::is_whitespace) {
            let t = w.trim();
            if !t.is_empty() {
                let lead = w.len() - w.trim_start().len();
                names.push((t.to_owned(), (value_at + off + lead, value_at + off + lead + t.len())));
            }
            off += w.len();
        }
        let misplaced = if self.stack.len() > 1 {
            Some(if self.stack[..self.stack.len() - 1].iter().any(|b| b.kind == Kind::Group) { "it is inside an at-rule" } else { "it is inside a nested rule" }.to_owned())
        } else if !self.stack.last().is_some_and(|b| b.class.is_some() && b.lone) {
            Some("its selector is not a single class".to_owned())
        } else {
            None
        };
        self.out.composes.push(Compose { span: (at, at + stmt.len()), names, from, misplaced });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(f: &ModuleFacts) -> Vec<&str> {
        f.classes.iter().map(|c| c.name.as_str()).collect()
    }

    #[test]
    fn locals_conventions() {
        assert_eq!(js_names("btn-primary", "asIs"), ["btn-primary"]);
        assert_eq!(js_names("btn-primary", "camelCase"), ["btn-primary", "btnPrimary"]);
        assert_eq!(js_names("btn_primary-x", "camelCaseOnly"), ["btnPrimaryX"]);
        assert_eq!(js_names("btn_primary-x", "dashes"), ["btn_primary-x", "btn_primaryX"]);
        assert_eq!(js_names("a-b", "dashesOnly"), ["aB"]);
    }

    #[test]
    fn local_and_global_classes() {
        let f = scan(".a { x: y } :global(.b) .c { } :global .d .e {} .f :global(.g) {} :local(.h) {}\n.i, .j { }", false);
        assert_eq!(names(&f), ["a", "c", "f", "h", "i", "j"]);
        assert!(!f.incomplete);
    }

    #[test]
    fn lone_rules_can_be_removed() {
        let src = ".a {\n  color: red;\n}\n.b { .c { } }\n.d, .e {}\n";
        let f = scan(src, false);
        let a = f.classes.iter().find(|c| c.name == "a").unwrap();
        assert_eq!(&src[a.rule.unwrap().0..a.rule.unwrap().1], ".a {\n  color: red;\n}\n");
        assert!(f.classes.iter().find(|c| c.name == "b").unwrap().rule.is_none());
        assert!(f.classes.iter().find(|c| c.name == "d").unwrap().rule.is_none());
    }

    #[test]
    fn composes_exports_and_values() {
        let src = ".a { composes: b c; }\n.d { composes: e from \"./x.module.css\"; composes: f from global; }\n:export { k: v; }\n@value primary: red;\n@value small, large from \"./v.css\";\n";
        let f = scan(src, false);
        assert_eq!(f.composes.len(), 3);
        assert_eq!(f.composes[0].names.iter().map(|n| n.0.as_str()).collect::<Vec<_>>(), ["b", "c"]);
        assert_eq!(f.composes[1].from, ComposeFrom::File("./x.module.css".into()));
        assert_eq!(f.composes[2].from, ComposeFrom::Global);
        assert!(f.composes.iter().all(|c| c.misplaced.is_none()));
        assert_eq!(f.exports, ["k", "large", "primary", "small"]);
    }

    #[test]
    fn composes_in_the_wrong_place() {
        let f = scan(".a, .b { composes: c; }\n.d { .e { composes: f; } }\n@media print { .g { composes: h; } }\n", false);
        let why: Vec<Option<&str>> = f.composes.iter().map(|c| c.misplaced.as_deref()).collect();
        assert_eq!(why[0], Some("its selector is not a single class"));
        assert_eq!(why[1], Some("it is inside a nested rule"));
        assert_eq!(why[2], Some("it is inside an at-rule"));
    }

    #[test]
    fn sass_parent_suffixes() {
        let f = scan(".btn { &-primary { a: b } &__icon { } & + & { } }\n.x, .y { &-z { } }\n", true);
        assert_eq!(names(&f), ["btn", "btn-primary", "btn__icon", "x", "y"]);
        assert!(f.incomplete, "a suffix on a selector list is not named");
    }

    #[test]
    fn keyframes_comments_strings_and_urls() {
        let f = scan("@keyframes k { from { a: b } }\n/* .no */ .a { content: \".no\"; background: url(.no) }\n// .no\n.b { }", true);
        assert_eq!(names(&f), ["a", "b"]);
        let css = scan("// not a comment\n.a { }", false);
        assert_eq!(names(&css), ["a"]);
    }
}
