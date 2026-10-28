//! The CSS model the `css/*`, `css-modules/*` and `scss/*` native rules
//! read: statements (rules, at-rules, declarations, comments), component
//! values, selectors and media queries, every piece with its byte range in the
//! source. Since W-62 the statements, each rule's selector list, each
//! declaration's and at-rule's component values, the media queries and the
//! selectors of an `@page` or `@scope` prelude come from the file's
//! tree-sitter tree ([`css_tree`](crate::css_tree); SR-ARCHITECTURE decision
//! 15b); this module's text reader builds the same sheet when the tree holds
//! an error (the CSS the grammar rejects) and reads what the tree does not
//! give the rules yet: the selectors inside a pseudo-class's arguments,
//! keyframe selectors, a font list split at its commas, and selectors a rule
//! builds itself. The grammar still verifies every fix the rules make
//! (decision 5).
//!
//! It never fails: anything it can't place becomes an `Other` statement,
//! and the rules skip what they don't recognise.

use std::cell::RefCell;
use std::rc::Rc;

pub type Range = (usize, usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Rule,
    AtRule,
    Decl,
    Comment,
    /// Text that is neither: no colon, no block.
    Other,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    /// The whole statement, without a trailing `;`.
    pub start: usize,
    pub end: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    /// Rule: the selector. At-rule: the name, without `@`. Declaration: the
    /// property. Trimmed.
    pub name: Range,
    /// At-rule: the prelude. Declaration: the value, without `!important`.
    /// Trimmed.
    pub params: Range,
    /// Declaration: the `!…` annotation, from `!` to its end.
    pub important: Option<Range>,
    /// The `{` and the `}` (or the end of the source when unclosed).
    pub block: Option<Range>,
    /// Rule: its selector list; `@page` and `@scope`: their prelude's
    /// selectors. When read from the tree
    /// ([`css_tree::selectors`](crate::css_tree::selectors)); see [`Sheet::selectors`].
    pub selectors: Option<Vec<Complex>>,
    /// Declaration: its value's component values; at-rule (Sass's included):
    /// its prelude's. When read from the tree ([`css_tree`](crate::css_tree));
    /// see [`Sheet::values`].
    pub values: Option<Vec<VNode>>,
    /// `@media` and `@custom-media`: the prelude's media queries, when read
    /// from the tree; see [`MediaQuery`].
    pub queries: Option<Vec<MediaQuery>>,
    /// The statement ends with `;`, at this offset.
    pub semicolon: Option<usize>,
}

#[derive(Debug, Default)]
pub struct Sheet {
    pub nodes: Vec<Node>,
    pub top: Vec<usize>,
    /// Read as SCSS (`//` comments blanked, Sass statements kept apart).
    pub scss: bool,
    /// Read as a CSS Module: `@value`, `composes:`, `:export {}` and
    /// `:import() {}` are kept apart, `:global` and `:local` are known.
    pub module: bool,
}

impl Sheet {
    /// The ancestors of a node, nearest first.
    pub fn ancestors(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        std::iter::successors(self.nodes[i].parent, move |&p| self.nodes[p].parent)
    }
    /// The at-rule name of a node, lowercased, without a vendor prefix.
    pub fn at_name(&self, i: usize, src: &str) -> String {
        unprefixed(&src[self.nodes[i].name.0..self.nodes[i].name.1].to_ascii_lowercase()).to_owned()
    }
    /// Whether a rule is a keyframe block: its parent is `@keyframes`.
    pub fn in_keyframes(&self, i: usize, src: &str) -> bool {
        self.nodes[i].parent.is_some_and(|p| self.nodes[p].kind == Kind::AtRule && self.at_name(p, src) == "keyframes")
    }
    /// The nearest enclosing at-rule. In SCSS a Sass at-rule's block
    /// (`@include x { … }`, `@if`, `@each`, a mixin's body) is an `Other`
    /// node, and a context of its own: `.a` inside `@include up(sm) { … }`
    /// does not repeat a top-level `.a` (W-62).
    pub fn at_rule_context(&self, i: usize) -> Option<usize> {
        let mut p = self.nodes[i].parent;
        while let Some(x) = p {
            match self.nodes[x].kind {
                Kind::AtRule | Kind::Other => return Some(x),
                Kind::Rule => p = self.nodes[x].parent,
                _ => return None,
            }
        }
        None
    }
    /// Every node in source order.
    pub fn walk(&self) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.nodes.len());
        fn go(s: &Sheet, list: &[usize], out: &mut Vec<usize>) {
            for &i in list {
                out.push(i);
                go(s, &s.nodes[i].children, out);
            }
        }
        go(self, &self.top, &mut out);
        out
    }
    /// A rule's selector list, or the selectors of an `@page` or `@scope`
    /// prelude: from the tree when the sheet was read from one, otherwise
    /// read from its text (the two agree on every stylesheet the tree reads:
    /// W-62; a prelude's text reading also holds its parentheses and `to`,
    /// which no rule reads).
    pub fn selectors(&self, i: usize, src: &str) -> std::borrow::Cow<'_, [Complex]> {
        let n = &self.nodes[i];
        let r = if n.kind == Kind::AtRule { n.params } else { n.name };
        match &n.selectors {
            Some(list) => std::borrow::Cow::Borrowed(list),
            None => std::borrow::Cow::Owned(selector_list(src, r.0, r.1)),
        }
    }
    /// A statement's component values: a declaration's value or an at-rule's
    /// prelude from the tree when the sheet was read from one, otherwise its
    /// `params` read from the text (the two agree on every stylesheet the
    /// tree reads: W-62).
    pub fn values(&self, i: usize, src: &str) -> std::borrow::Cow<'_, [VNode]> {
        let n = &self.nodes[i];
        match &n.values {
            Some(list) => std::borrow::Cow::Borrowed(list),
            None => std::borrow::Cow::Owned(values(src, n.params.0, n.params.1)),
        }
    }
    pub fn decls(&self) -> impl Iterator<Item = usize> + '_ {
        self.walk().into_iter().filter(|&i| self.nodes[i].kind == Kind::Decl)
    }
}

pub fn text(src: &str, r: Range) -> &str {
    &src[r.0..r.1]
}

/// `-webkit-foo` → `foo`; anything else unchanged.
pub fn unprefixed(name: &str) -> &str {
    vendor_prefix(name).map_or(name, |p| &name[p.len()..])
}

/// The vendor prefix of a name (`-webkit-`), if it has one.
pub fn vendor_prefix(name: &str) -> Option<&str> {
    let rest = name.strip_prefix('-')?;
    if rest.starts_with('-') {
        return None;
    }
    let dash = rest.find('-')?;
    (dash > 0).then(|| &name[..dash + 2])
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b >= 0x80
}

/// The end of an identifier starting at `i` (escapes included).
pub fn ident_end(src: &str, mut i: usize) -> usize {
    let b = src.as_bytes();
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() {
            i += 1 + src[i + 1..].chars().next().map_or(1, char::len_utf8);
        } else if is_ident_byte(b[i]) {
            i += 1;
        } else {
            break;
        }
    }
    i
}

/// Skip a string starting at the quote at `i`; returns the offset past
/// it. Like PostCSS, a string runs to its closing quote even across a
/// line break, so one bad string doesn't swallow the statements after it.
pub fn skip_string(b: &[u8], i: usize) -> usize {
    let q = b[i];
    let mut j = i + 1;
    while j < b.len() {
        match b[j] {
            b'\\' => j += 2,
            c if c == q => return j + 1,
            _ => j += 1,
        }
    }
    b.len()
}

/// Skip a comment starting at `/*` at `i`.
pub fn skip_comment(src: &str, i: usize) -> usize {
    src[i + 2..].find("*/").map_or(src.len(), |e| i + 2 + e + 2)
}

/// Skip an SCSS interpolation `#{…}` starting at the `#` at `i`; returns the
/// offset past its closing brace (strings inside it are skipped whole).
pub fn skip_interpolation(b: &[u8], i: usize) -> usize {
    let mut depth = 0usize;
    let mut j = i + 1;
    while j < b.len() {
        match b[j] {
            b'"' | b'\'' => {
                j = skip_string(b, j);
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            _ => {}
        }
        j += 1;
    }
    b.len()
}

/// Scan from `i` to the first of `stops` outside strings, comments,
/// parentheses and brackets (and braces, when `braces` is set). Returns
/// the offset and the byte found (0 at the end).
fn scan(src: &str, mut i: usize, end: usize, stops: &[u8], braces: bool) -> (usize, u8) {
    let b = src.as_bytes();
    let mut depth = 0usize;
    while i < end {
        let c = b[i];
        match c {
            b'"' | b'\'' => {
                i = skip_string(b, i).min(end);
                continue;
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i = skip_comment(src, i).min(end);
                continue;
            }
            b'\\' => {
                i += 2;
                continue;
            }
            // SCSS interpolation `#{…}` is one unit, whatever it holds.
            b'#' if b.get(i + 1) == Some(&b'{') => {
                i = skip_interpolation(b, i).min(end);
                continue;
            }
            b'(' | b'[' => depth += 1,
            b'{' if braces && depth > 0 => depth += 1,
            b')' | b']' => depth = depth.saturating_sub(1),
            b'}' if braces && depth > 0 => {
                depth -= 1;
                i += 1;
                continue;
            }
            _ => {}
        }
        if depth == 0 && stops.contains(&c) {
            if braces && c == b'{' {
                depth += 1;
                i += 1;
                continue;
            }
            return (i, c);
        }
        i += 1;
    }
    (end.min(b.len()), 0)
}

pub fn trim(src: &str, start: usize, end: usize) -> Range {
    let s = &src[start..end];
    let lead = s.len() - s.trim_start().len();
    let t = s.trim();
    (start + lead, start + lead + t.len())
}

/// Trim whitespace and comments at both ends of a range.
pub fn trim_comments(src: &str, r: Range) -> Range {
    let (mut s, mut e) = trim(src, r.0, r.1);
    loop {
        let t = &src[s..e];
        if t.starts_with("/*") {
            let c = skip_comment(src, s).min(e);
            (s, e) = trim(src, c, e);
        } else if t.ends_with("*/") && t.len() >= 4 {
            match src[s..e - 2].rfind("/*") {
                Some(o) => (s, e) = trim(src, s, s + o),
                None => break,
            }
        } else {
            break;
        }
    }
    (s, e)
}

struct Parser<'s> {
    src: &'s str,
    nodes: Vec<Node>,
    scss: bool,
    module: bool,
}

/// Whether a rule prelude opens an ICSS block (`:export`, `:import("x")`),
/// whose declarations are not CSS properties.
fn icss_block(prelude: &str) -> bool {
    let t = prelude.trim();
    t == ":export" || t.starts_with(":import(") || t.starts_with(":import (")
}

/// The at-rules only Sass has. In an SCSS sheet they become `Other`
/// statements (with their blocks), so no `css/*` rule judges Sass as CSS;
/// the `scss/*` rules read them.
pub const SASS_AT_RULES: &[&str] = &[
    "mixin", "include", "function", "return", "if", "else", "each", "for", "while", "use", "forward", "extend", "debug", "warn", "error",
    "content", "at-root", "import",
];

/// Whether a rule prelude is a Sass nested property (`font: {` or
/// `font: bold {`): a colon followed by a space or the end.
fn nested_property(prelude: &str) -> bool {
    let t = prelude.trim_end();
    if t.starts_with(['&', '.', '#', '%', '[', '*', ':', '>', '+', '~', '@']) || t.contains("#{") {
        return false;
    }
    let Some(c) = t.find(':') else { return false };
    let (name, rest) = t.split_at(c);
    !name.is_empty() && name.bytes().all(is_ident_byte) && (rest.len() == 1 || rest.as_bytes().get(1).is_some_and(u8::is_ascii_whitespace))
}

impl Parser<'_> {
    fn push(&mut self, n: Node) -> usize {
        self.nodes.push(n);
        self.nodes.len() - 1
    }

    fn node(kind: Kind, start: usize, end: usize, parent: Option<usize>) -> Node {
        Node { kind, start, end, parent, children: Vec::new(), name: (start, start), params: (start, start), important: None, block: None, semicolon: None, selectors: None, values: None, queries: None }
    }

    /// Parse statements from `i` until a `}` (returned, not consumed) or the end.
    fn block(&mut self, mut i: usize, parent: Option<usize>) -> (Vec<usize>, usize) {
        let src = self.src;
        let b = src.as_bytes();
        let mut list = Vec::new();
        loop {
            while i < b.len() && (b[i].is_ascii_whitespace() || b[i] == b';') {
                i += 1;
            }
            if i >= b.len() {
                return (list, i);
            }
            if src[i..].starts_with("/*") {
                let e = skip_comment(src, i);
                let mut n = Self::node(Kind::Comment, i, e, parent);
                n.params = (i + 2, if src[i..e].ends_with("*/") && e - i >= 4 { e - 2 } else { e });
                list.push(self.push(n));
                i = e;
                continue;
            }
            if b[i] == b'}' {
                if parent.is_some() {
                    return (list, i);
                }
                i += 1;
                continue;
            }
            let start = i;
            if b[i] == b'@' {
                let name_end = ident_end(src, i + 1);
                let (stop_at, stop) = scan(src, name_end, b.len(), b"{;}", false);
                let lower_name = src[i + 1..name_end].to_ascii_lowercase();
                let sass = (self.scss && SASS_AT_RULES.contains(&lower_name.as_str())) || (self.module && lower_name == "value");
                let mut n = Self::node(if sass { Kind::Other } else { Kind::AtRule }, start, stop_at, parent);
                n.name = (i + 1, name_end);
                n.params = trim(src, name_end, stop_at);
                let id = self.push(n);
                list.push(id);
                i = self.finish(id, stop_at, stop);
                continue;
            }
            // A custom property's value may hold `{}` blocks.
            let custom = src[i..].starts_with("--") && {
                let e = ident_end(src, i);
                src[e..].trim_start().starts_with(':')
            };
            let (stop_at, stop) = scan(src, i, b.len(), b";{}", custom);
            if stop == b'{' {
                let nested = (self.scss && nested_property(&src[start..stop_at])) || (self.module && icss_block(&src[start..stop_at]));
                let mut n = Self::node(if nested { Kind::Other } else { Kind::Rule }, start, stop_at, parent);
                n.name = trim(src, start, stop_at);
                let id = self.push(n);
                list.push(id);
                i = self.finish(id, stop_at, stop);
                continue;
            }
            let (colon, found) = scan(src, start, stop_at, b":", false);
            let mut n = Self::node(if found == b':' { Kind::Decl } else { Kind::Other }, start, stop_at, parent);
            if found == b':' {
                n.name = trim(src, start, colon);
                if self.module && src[n.name.0..n.name.1].eq_ignore_ascii_case("composes") {
                    n.kind = Kind::Other;
                }
                let (vs, ve) = trim(src, colon + 1, stop_at);
                n.params = (vs, ve);
                if !src[start..].starts_with("--") {
                    // The last `!` at the top level starts the annotation.
                    let mut j = vs;
                    let mut bang = None;
                    while j < ve {
                        let (at, c) = scan(src, j, ve, b"!", false);
                        if c != b'!' {
                            break;
                        }
                        bang = Some(at);
                        j = at + 1;
                    }
                    if let Some(at) = bang {
                        let after = trim(src, at + 1, ve);
                        let word_end = ident_end(src, after.0);
                        if word_end == after.1 && word_end > after.0 {
                            n.important = Some((at, ve));
                            n.params = trim(src, vs, at);
                        }
                    }
                }
            }
            n.end = trim(src, start, stop_at).1;
            let id = self.push(n);
            list.push(id);
            if stop == b';' {
                self.nodes[id].semicolon = Some(stop_at);
                i = stop_at + 1;
            } else {
                i = stop_at;
            }
        }
    }

    /// Finish a rule or at-rule whose prelude ended at `at` with `stop`.
    fn finish(&mut self, id: usize, at: usize, stop: u8) -> usize {
        let len = self.src.len();
        match stop {
            b'{' => {
                let (kids, close) = self.block(at + 1, Some(id));
                // A Sass nested property block (`font: { family: x }`) holds
                // property suffixes, not declarations: its statements are not kept.
                let nested = (self.scss || self.module) && self.nodes[id].kind == Kind::Other && !self.src[self.nodes[id].start..].starts_with('@');
                let kids = if nested {
                    self.nodes.truncate(id + 1);
                    Vec::new()
                } else {
                    kids
                };
                let n = &mut self.nodes[id];
                n.children = kids;
                n.block = Some((at, close));
                n.end = (close + 1).min(len);
                (close + 1).min(len)
            }
            b';' => {
                let n = &mut self.nodes[id];
                n.semicolon = Some(at);
                n.end = trim(self.src, n.start, at).1;
                at + 1
            }
            _ => {
                let n = &mut self.nodes[id];
                n.end = trim(self.src, n.start, at).1;
                at
            }
        }
    }
}

#[cfg(test)]
pub fn parse(src: &str) -> Sheet {
    parse_with(src, false, false)
}

/// Parse a stylesheet in a mode: SCSS (`//` comments, Sass statements) and/or
/// CSS Module (`composes`, `@value`, ICSS blocks).
pub fn parse_with(src: &str, scss: bool, module: bool) -> Sheet {
    let view = if scss { scss_view(src) } else { src.to_owned() };
    let mut p = Parser { src: &view, nodes: Vec::new(), scss, module };
    let (top, _) = p.block(0, None);
    Sheet { nodes: p.nodes, top, scss, module }
}

/// Parse SCSS: `//` comments are blanked first (same byte length, so every
/// range is a range of `src`), Sass statements are kept as `Other`.
#[cfg(test)]
pub fn parse_scss(src: &str) -> Sheet {
    parse_with(src, true, false)
}

/// The byte ranges of the `//` comments of an SCSS source, each from its
/// `//` to the end of its line (not the newline). Strings, `/* */`
/// comments, unquoted `url(…)` and `#{…}` interpolation are not scanned.
pub fn scss_line_comments(src: &str) -> Vec<Range> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => i = skip_string(b, i),
            b'/' if b.get(i + 1) == Some(&b'*') => i = skip_comment(src, i),
            b'#' if b.get(i + 1) == Some(&b'{') => i = skip_interpolation(b, i),
            b'/' if b.get(i + 1) == Some(&b'/') => {
                let mut end = src[i..].find('\n').map_or(b.len(), |o| i + o);
                if end > i && b[end - 1] == b'\r' {
                    end -= 1;
                }
                out.push((i, end));
                i = end;
            }
            b'u' | b'U' if src[i..].len() >= 4 && src[i..i + 4].eq_ignore_ascii_case("url(") && (i == 0 || !is_ident_byte(b[i - 1])) => {
                if src[i + 4..].trim_start().starts_with(['"', '\'']) {
                    i += 4;
                } else {
                    i = src[i..].find(')').map_or(b.len(), |o| i + o);
                }
            }
            _ => i += 1,
        }
    }
    out
}

/// `src` with every `//` comment replaced by spaces — the same length, the
/// same offsets.
pub fn scss_view(src: &str) -> String {
    let mut out = src.as_bytes().to_vec();
    for (s, e) in scss_line_comments(src) {
        out[s..e].fill(b' ');
    }
    String::from_utf8(out).expect("blanking whole comments keeps UTF-8")
}

/// The last parse: source, (scss, module, from the tree) flags, sheet.
type CachedSheet = (String, (bool, bool, bool), Rc<Sheet>);

thread_local! {
    static CACHE: RefCell<Option<CachedSheet>> = const { RefCell::new(None) };
    static VIEW: RefCell<Option<(String, Rc<str>)>> = const { RefCell::new(None) };
}

/// The text a CSS rule reads: an SCSS source with its `//` comments blanked
/// ([`scss_view`], same offsets), so a comment inside a selector list or a
/// value is not read as CSS (W-62: `.a, // note` gave `css/selector-type-no-unknown`
/// on the comment's words); a CSS source as it is. Computed once per file.
pub fn rule_source(src: &str, scss: bool) -> Rc<str> {
    if !scss {
        return Rc::from(src);
    }
    VIEW.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((s, view)) = c.as_ref()
            && s == src
        {
            return view.clone();
        }
        let view: Rc<str> = Rc::from(scss_view(src));
        *c = Some((src.to_owned(), view.clone()));
        view
    })
}

/// A fix whose edit covers part of an SCSS `//` comment is dropped (the
/// finding stays): the rule built its text from [`rule_source`], where the
/// comment is blanks, so the edit would delete it.
pub fn keep_line_comments(source: &str, out: &mut [crate::NativeFinding], from: usize) {
    let comments = scss_line_comments(source);
    if comments.is_empty() {
        return;
    }
    for f in &mut out[from..] {
        if f.edits.iter().any(|e| comments.iter().any(|&(s, end)| e.start < end && s < e.end.max(e.start + 1))) {
            f.edits.clear();
        }
    }
}

/// The sheet of a source (`scss`: read as SCSS), built from its tree `root`
/// ([`css_tree`](crate::css_tree)) or, when the tree holds an error or no
/// tree is given, from its text. Each file is read once, however many CSS
/// rules run on it. On a tree without an error the two readings are the same
/// sheet (W-62 checked it on every stylesheet of the pinned corpus), so the
/// rules that exist to report CSS the grammar rejects
/// (`css/media-query-no-invalid`, `css/at-rule-prelude-no-invalid`) read the
/// tree's statements too; what they judge, the prelude, they read as text.
pub fn sheet_for(root: Option<tree_sitter::Node<'_>>, src: &str, scss: bool, module: bool) -> Rc<Sheet> {
    let key = (scss, module, root.is_some());
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((s, k, sheet)) = c.as_ref()
            && *k == key
            && s == src
        {
            return sheet.clone();
        }
        let from_tree = root.and_then(|r| crate::css_tree::sheet(r, &rule_source(src, scss), scss, module));
        let sheet = Rc::new(from_tree.unwrap_or_else(|| parse_with(src, scss, module)));
        *c = Some((src.to_owned(), key, sheet.clone()));
        sheet
    })
}

/// Whether the file a native rule checks is a CSS Module: named
/// `*.module.css` or `*.module.scss`, or matching a glob of `[css] modules`
/// (the `modules` option, read from `[css]`).
pub fn is_module(ctx: &crate::NativeContext<'_>) -> bool {
    module_path(ctx.file.path(), ctx.options.get("modules"))
}

/// [`is_module`] for a path and the `modules` option.
pub fn module_path(path: &str, modules: Option<&serde_json::Value>) -> bool {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".module.css") || lower.ends_with(".module.scss") {
        return true;
    }
    let Some(globs) = modules.and_then(|v| v.as_array()) else { return false };
    let mut b = globset::GlobSetBuilder::new();
    for g in globs.iter().filter_map(|g| g.as_str()) {
        if let Ok(g) = globset::GlobBuilder::new(g.trim_start_matches("./")).literal_separator(false).build() {
            b.add(g);
        }
    }
    b.build().is_ok_and(|set| set.is_match(path))
}

// ---------------------------------------------------------------------
// Component values

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VKind {
    Word,
    Str,
    Func,
    /// `,`, `/` or `:`.
    Div,
    Space,
    Comment,
}

#[derive(Clone, Debug)]
pub struct VNode {
    pub kind: VKind,
    pub start: usize,
    pub end: usize,
    /// Func: the name's range (empty for a bare `(…)`).
    pub name: Range,
    /// Func: inside the parentheses.
    pub inner: Range,
    pub nodes: Vec<VNode>,
}

impl VNode {
    pub fn text<'s>(&self, src: &'s str) -> &'s str {
        &src[self.start..self.end]
    }
    pub fn name<'s>(&self, src: &'s str) -> &'s str {
        &src[self.name.0..self.name.1]
    }
    /// Every node, depth first; `f` returns false to skip a function's arguments.
    pub fn walk<'a>(list: &'a [VNode], f: &mut dyn FnMut(&'a VNode) -> bool) {
        for n in list {
            if f(n) && n.kind == VKind::Func {
                VNode::walk(&n.nodes, f);
            }
        }
    }
}

/// Parse a value range into component values.
pub fn values(src: &str, start: usize, end: usize) -> Vec<VNode> {
    let mut i = start;
    values_until(src, &mut i, end, false)
}

fn values_until(src: &str, i: &mut usize, end: usize, in_func: bool) -> Vec<VNode> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let leaf = |kind, s, e| VNode { kind, start: s, end: e, name: (s, s), inner: (s, s), nodes: Vec::new() };
    while *i < end {
        let s = *i;
        let c = b[s];
        if c.is_ascii_whitespace() {
            while *i < end && b[*i].is_ascii_whitespace() {
                *i += 1;
            }
            out.push(leaf(VKind::Space, s, *i));
        } else if src[s..].starts_with("/*") {
            *i = skip_comment(src, s).min(end);
            out.push(leaf(VKind::Comment, s, *i));
        } else if c == b'"' || c == b'\'' {
            *i = skip_string(b, s).min(end);
            out.push(leaf(VKind::Str, s, *i));
        } else if matches!(c, b',' | b'/' | b':') {
            *i += 1;
            out.push(leaf(VKind::Div, s, *i));
        } else if c == b')' {
            if in_func {
                return out;
            }
            *i += 1;
            out.push(leaf(VKind::Word, s, *i));
        } else if c == b'(' {
            out.push(func(src, i, end, s, s));
        } else {
            let mut j = s;
            while j < end {
                let d = b[j];
                if d.is_ascii_whitespace() || matches!(d, b'"' | b'\'' | b',' | b'/' | b':' | b'(' | b')') {
                    break;
                }
                if d == b'\\' {
                    j += 2;
                    continue;
                }
                j += 1;
            }
            let j = j.min(end);
            if j < end && b[j] == b'(' {
                *i = j;
                out.push(func(src, i, end, s, j));
            } else {
                *i = j.max(s + 1);
                out.push(leaf(VKind::Word, s, *i));
            }
        }
    }
    out
}

fn func(src: &str, i: &mut usize, end: usize, start: usize, open: usize) -> VNode {
    let name = &src[start..open];
    *i = open + 1;
    let inner_start = *i;
    let nodes = if name.eq_ignore_ascii_case("url") && !src[inner_start..end].trim_start().starts_with(['"', '\'']) {
        // An unquoted URL is one word.
        let close = src[inner_start..end].find(')').map_or(end, |o| inner_start + o);
        *i = close;
        vec![VNode { kind: VKind::Word, start: inner_start, end: close, name: (inner_start, inner_start), inner: (inner_start, inner_start), nodes: Vec::new() }]
    } else {
        values_until(src, i, end, true)
    };
    let inner_end = *i;
    if *i < end && src.as_bytes()[*i] == b')' {
        *i += 1;
    }
    VNode { kind: VKind::Func, start, end: *i, name: (start, open), inner: (inner_start, inner_end), nodes }
}

/// The parts of the values `list` between its top-level commas, each trimmed
/// of white space and comments (an empty part is empty where its comma is),
/// as [`split_commas`] reads them from the text of `r`, the range the values
/// cover.
pub fn comma_parts(list: &[VNode], src: &str, r: Range) -> Vec<Range> {
    let mut out = Vec::new();
    let mut part: Option<Range> = None;
    for n in list {
        match n.kind {
            VKind::Div if n.text(src) == "," => out.push(part.take().unwrap_or((n.start, n.start))),
            VKind::Space | VKind::Comment => {}
            _ => part = Some(part.map_or((n.start, n.end), |p| (p.0, n.end))),
        }
    }
    out.push(part.unwrap_or((r.1, r.1)));
    out
}

/// The non-space, non-comment nodes of a list.
pub fn significant(list: &[VNode]) -> Vec<&VNode> {
    list.iter().filter(|n| !matches!(n.kind, VKind::Space | VKind::Comment)).collect()
}

/// A number and its unit (`%` included), when a word is a dimension.
pub fn dimension(word: &str) -> Option<(&str, &str)> {
    let b = word.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let digits_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let mut any = i > digits_start;
    if i < b.len() && b[i] == b'.' && i + 1 < b.len() && b[i + 1].is_ascii_digit() {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        any = true;
    }
    if !any {
        return None;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        if j < b.len() && b[j].is_ascii_digit() {
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            i = j;
        }
    }
    let unit = &word[i..];
    if unit.is_empty() || unit == "%" || (unit.as_bytes()[0].is_ascii_alphabetic() && unit.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')) {
        Some((&word[..i], unit))
    } else {
        None
    }
}

// ---------------------------------------------------------------------
// Media queries

/// One media query of an `@media` or `@custom-media` prelude: its range
/// (between the commas, trimmed), its media types and its features. The text
/// reader is `css_media`'s, which also judges whether a query is valid.
#[derive(Clone, Debug, PartialEq)]
pub struct MediaQuery {
    pub range: Range,
    pub types: Vec<Range>,
    pub features: Vec<Feature>,
}

/// A media feature inside parentheses.
#[derive(Clone, Debug, PartialEq)]
pub struct Feature {
    pub whole: Range,
    pub name: Range,
    /// `name: value`.
    pub value: Option<Range>,
    /// Values of a range form (`width >= 600px`, `1px < width < 2px`).
    pub range_values: Vec<Range>,
}

// ---------------------------------------------------------------------
// Selectors

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SKind {
    Type,
    Universal,
    Class,
    Id,
    Attribute,
    PseudoClass,
    PseudoElement,
    Nesting,
    Other,
}

#[derive(Clone, Debug)]
pub struct Simple {
    pub kind: SKind,
    pub start: usize,
    pub end: usize,
    /// The name, without `.`, `#`, `:` or `::` (type: with any namespace).
    pub name: Range,
    /// Inside the parentheses of a functional pseudo-class or element.
    pub args: Option<Range>,
}

#[derive(Clone, Debug)]
pub struct Compound {
    pub start: usize,
    pub end: usize,
    pub parts: Vec<Simple>,
}

#[derive(Clone, Debug)]
pub struct Complex {
    pub start: usize,
    pub end: usize,
    pub compounds: Vec<Compound>,
    /// The combinator before each compound after the first (`" "` for a
    /// descendant), and a leading one for a relative selector.
    pub combinators: Vec<String>,
    pub leading: Option<String>,
}

impl Complex {
    pub fn simples(&self) -> impl Iterator<Item = &Simple> {
        self.compounds.iter().flat_map(|c| c.parts.iter())
    }
}

/// Split a range at top-level commas.
pub fn split_commas(src: &str, start: usize, end: usize) -> Vec<Range> {
    let mut out = Vec::new();
    let mut i = start;
    loop {
        let (at, c) = scan(src, i, end, b",", false);
        out.push(trim_comments(src, (i, at)));
        if c != b',' {
            break;
        }
        i = at + 1;
    }
    out
}

/// Parse a selector list.
pub fn selector_list(src: &str, start: usize, end: usize) -> Vec<Complex> {
    split_commas(src, start, end).into_iter().map(|r| complex(src, r.0, r.1)).collect()
}

pub fn complex(src: &str, start: usize, end: usize) -> Complex {
    let b = src.as_bytes();
    let mut out = Complex { start, end, compounds: Vec::new(), combinators: Vec::new(), leading: None };
    let mut cur: Option<Compound> = None;
    let mut pending: Option<String> = None;
    let mut i = start;
    let flush = |cur: &mut Option<Compound>, out: &mut Complex, pending: &mut Option<String>| {
        if let Some(c) = cur.take() {
            if out.compounds.is_empty() {
                out.leading = pending.take().filter(|p| p != " ");
            } else {
                out.combinators.push(pending.take().unwrap_or_else(|| " ".to_owned()));
            }
            out.compounds.push(c);
        }
    };
    while i < end {
        let c = b[i];
        if c.is_ascii_whitespace() || src[i..].starts_with("/*") {
            if src[i..].starts_with("/*") {
                i = skip_comment(src, i).min(end);
            } else {
                i += 1;
            }
            if cur.is_some() {
                flush(&mut cur, &mut out, &mut pending);
                pending = Some(" ".to_owned());
            }
            continue;
        }
        if matches!(c, b'>' | b'+' | b'~') || (c == b'|' && b.get(i + 1) == Some(&b'|')) {
            flush(&mut cur, &mut out, &mut pending);
            let len = if c == b'|' { 2 } else { 1 };
            pending = Some(src[i..i + len].to_owned());
            i += len;
            continue;
        }
        let s = i;
        let simple = match c {
            b'.' | b'#' => {
                let e = ident_end(src, i + 1).min(end);
                i = e.max(i + 1);
                Simple { kind: if c == b'.' { SKind::Class } else { SKind::Id }, start: s, end: i, name: (s + 1, e), args: None }
            }
            b'[' => {
                let (close, _) = scan(src, i + 1, end, b"]", false);
                i = (close + 1).min(end);
                let inner = trim(src, s + 1, close);
                let ne = ident_end(src, inner.0).min(close);
                Simple { kind: SKind::Attribute, start: s, end: i, name: (inner.0, ne), args: Some(inner) }
            }
            b':' => {
                let element = b.get(i + 1) == Some(&b':');
                let ns = if element { i + 2 } else { i + 1 };
                let e = ident_end(src, ns).min(end);
                let mut args = None;
                i = e;
                if i < end && b[i] == b'(' {
                    let (close, _) = scan(src, i + 1, end, b")", false);
                    args = Some((i + 1, close));
                    i = (close + 1).min(end);
                }
                if i == s + 1 {
                    i += 1;
                }
                Simple { kind: if element { SKind::PseudoElement } else { SKind::PseudoClass }, start: s, end: i, name: (ns, e), args }
            }
            b'&' => {
                // Sass's parent suffix (`&-wide`, `&__item`) is part of the nesting selector.
                i = ident_end(src, i + 1).min(end);
                Simple { kind: SKind::Nesting, start: s, end: i, name: (s, i), args: None }
            }
            b'*' => {
                i += 1;
                if i < end && b[i] == b'|' && b.get(i + 1) != Some(&b'|') {
                    let e = ident_end(src, i + 1).min(end);
                    let star = b.get(i + 1) == Some(&b'*');
                    i = if star { i + 2 } else { e };
                    if !star {
                        let s2 = Simple { kind: SKind::Type, start: s, end: i, name: (s, i), args: None };
                        cur.get_or_insert(Compound { start: s, end: i, parts: Vec::new() }).parts.push(s2);
                        if let Some(c) = cur.as_mut() {
                            c.end = i;
                        }
                        continue;
                    }
                }
                Simple { kind: SKind::Universal, start: s, end: i, name: (s, i), args: None }
            }
            _ if is_ident_byte(c) || c == b'\\' || c == b'|' => {
                let mut e = ident_end(src, i).min(end);
                if e < end && b[e] == b'|' && b.get(e + 1) != Some(&b'|') {
                    if b.get(e + 1) == Some(&b'*') {
                        i = e + 2;
                        let s2 = Simple { kind: SKind::Universal, start: s, end: i, name: (s, i), args: None };
                        cur.get_or_insert(Compound { start: s, end: i, parts: Vec::new() }).parts.push(s2);
                        if let Some(c) = cur.as_mut() {
                            c.end = i;
                        }
                        continue;
                    }
                    e = ident_end(src, e + 1).min(end);
                }
                i = e.max(i + 1);
                Simple { kind: SKind::Type, start: s, end: i, name: (s, i), args: None }
            }
            _ => {
                i += src[i..].chars().next().map_or(1, char::len_utf8);
                Simple { kind: SKind::Other, start: s, end: i, name: (s, i), args: None }
            }
        };
        let comp = cur.get_or_insert(Compound { start: s, end: i, parts: Vec::new() });
        comp.end = i;
        comp.parts.push(simple);
    }
    flush(&mut cur, &mut out, &mut pending);
    out
}

/// Every simple selector of a list, descending into the arguments of
/// pseudo-classes that take selectors.
pub fn all_simples(src: &str, list: &[Complex], out: &mut Vec<Simple>) {
    for c in list {
        for s in c.simples() {
            out.push(s.clone());
            if let Some((a, b)) = s.args
                && matches!(s.kind, SKind::PseudoClass | SKind::PseudoElement)
            {
                let name = text(src, s.name).to_ascii_lowercase();
                let name = unprefixed(&name);
                if takes_selectors(name) {
                    all_simples(src, &selector_list(src, a, b), out);
                } else if matches!(name, "nth-child" | "nth-last-child")
                    && let Some(of) = find_of(src, a, b)
                {
                    all_simples(src, &selector_list(src, of, b), out);
                }
            }
        }
    }
}

/// Pseudo-classes and elements whose argument is a selector list.
pub fn takes_selectors(name: &str) -> bool {
    matches!(name, "not" | "is" | "where" | "has" | "matches" | "any" | "host" | "host-context" | "slotted" | "cue" | "current" | "past" | "future")
}

/// The start of the selector after ` of ` in an `nth-child()` argument.
pub fn find_of(src: &str, a: usize, b: usize) -> Option<usize> {
    let t = &src[a..b];
    let lower = t.to_ascii_lowercase();
    let mut from = 0;
    while let Some(p) = lower[from..].find("of") {
        let p = from + p;
        let before = p == 0 || lower.as_bytes()[p - 1].is_ascii_whitespace();
        let after = lower.as_bytes().get(p + 2).is_none_or(|c| c.is_ascii_whitespace());
        if before && after && p > 0 {
            return Some(a + p + 2);
        }
        from = p + 2;
    }
    None
}

/// A selector's text with whitespace collapsed and spaces around
/// combinators and commas removed, for comparing.
pub fn normalize_selector(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    let mut space = false;
    while i < b.len() {
        let c = b[i];
        if s[i..].starts_with("/*") {
            i = skip_comment(s, i);
            space = true;
            continue;
        }
        if c == b'"' || c == b'\'' {
            let e = skip_string(b, i);
            if space && !out.is_empty() {
                out.push(' ');
            }
            space = false;
            out.push_str(&s[i..e]);
            i = e;
            continue;
        }
        if c.is_ascii_whitespace() {
            space = true;
            i += 1;
            continue;
        }
        let ch = s[i..].chars().next().unwrap_or(' ');
        if matches!(c, b'>' | b'+' | b'~' | b',' | b'(' | b')' | b'[' | b']' | b'=') {
            space = false;
            while out.ends_with(' ') {
                out.pop();
            }
            out.push(ch);
        } else {
            if space && !out.is_empty() && !out.ends_with(['>', '+', '~', ',', '(', '[', '=']) {
                out.push(' ');
            }
            space = false;
            out.push(ch);
        }
        i += ch.len_utf8();
    }
    out
}

/// Specificity `(ids, classes, types)` of a complex selector.
pub fn specificity(src: &str, c: &Complex, nesting: (u32, u32, u32)) -> (u32, u32, u32) {
    let mut t = (0, 0, 0);
    let add = |t: &mut (u32, u32, u32), s: (u32, u32, u32)| {
        t.0 += s.0;
        t.1 += s.1;
        t.2 += s.2;
    };
    for s in c.simples() {
        let name = text(src, s.name).to_ascii_lowercase();
        let name = unprefixed(&name).to_owned();
        match s.kind {
            SKind::Id => t.0 += 1,
            SKind::Class | SKind::Attribute => t.1 += 1,
            SKind::Type => t.2 += 1,
            SKind::PseudoElement => t.2 += 1,
            SKind::Nesting => add(&mut t, nesting),
            SKind::PseudoClass => {
                if matches!(name.as_str(), "before" | "after" | "first-line" | "first-letter") {
                    t.2 += 1;
                } else if name == "where" {
                } else if let (true, Some((a, b))) = (matches!(name.as_str(), "is" | "not" | "has" | "matches" | "any"), s.args) {
                    add(&mut t, max_specificity(src, &selector_list(src, a, b), nesting));
                } else if let (true, Some((a, b))) = (matches!(name.as_str(), "nth-child" | "nth-last-child"), s.args) {
                    t.1 += 1;
                    if let Some(of) = find_of(src, a, b) {
                        add(&mut t, max_specificity(src, &selector_list(src, of, b), nesting));
                    }
                } else {
                    t.1 += 1;
                }
            }
            SKind::Universal | SKind::Other => {}
        }
    }
    t
}

pub fn max_specificity(src: &str, list: &[Complex], nesting: (u32, u32, u32)) -> (u32, u32, u32) {
    list.iter().map(|c| specificity(src, c, nesting)).max().unwrap_or((0, 0, 0))
}

/// Whether a selector text uses syntax from a preprocessor or template.
pub fn non_standard(s: &str) -> bool {
    s.contains("#{") || s.contains("${") || s.contains("@{") || s.contains("{{") || s.contains("<%") || s.starts_with('%')
}

// ---------------------------------------------------------------------
// Data tables

use serde_json::Value;
use std::collections::BTreeSet;

/// A list in a data table (`table` is the table's name, `key` a field of
/// it, or `""` for a table that is itself a list), as a lowercased set.
pub fn set(data: &std::collections::BTreeMap<String, std::sync::Arc<Value>>, table: &str, key: &str) -> BTreeSet<String> {
    let v = data.get(table).map(|v| v.as_ref());
    let v = if key.is_empty() { v } else { v.and_then(|v| v.get(key)) };
    match v {
        Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(|s| s.to_ascii_lowercase())).collect(),
        Some(Value::Object(o)) => o.keys().map(|s| s.to_ascii_lowercase()).collect(),
        _ => BTreeSet::new(),
    }
}

pub fn table<'d>(data: &'d std::collections::BTreeMap<String, std::sync::Arc<Value>>, table: &str) -> Option<&'d Value> {
    data.get(table).map(|v| v.as_ref())
}

// ---------------------------------------------------------------------
// Rule plumbing

/// Declare a CSS native rule: it runs once per stylesheet, on its root.
macro_rules! css_rule {
    ($(#[$m:meta])* $name:ident, $id:literal, |$sheet:ident, $src:ident, $ctx:ident, $out:ident| $body:block) => {
        $(#[$m])*
        pub struct $name;
        impl $crate::NativeRule for $name {
            fn id(&self) -> &'static str {
                $id
            }
            fn languages(&self) -> &'static [codelaw_core::Language] {
                &[codelaw_core::Language::Css, codelaw_core::Language::Scss]
            }
            fn kinds(&self) -> &'static [&'static str] {
                // The root is an `ERROR` node when the grammar cannot recover:
                // the rules then read the statements from the text, so they
                // still run, once, on the root.
                &["stylesheet", "ERROR"]
            }
            /// `[css]`'s `modules` globs name the CSS Modules (W-63).
            fn config_section(&self) -> Option<&'static str> {
                Some("css")
            }
            #[allow(unused_variables)]
            fn check(&self, node: codelaw_lang::SyntaxNode<'_>, $ctx: &$crate::NativeContext<'_>, $out: &mut Vec<$crate::NativeFinding>) {
                if node.parent().is_some() {
                    return;
                }
                let scss = $ctx.language == codelaw_core::Language::Scss;
                let view = $crate::css_syntax::rule_source($ctx.source, scss);
                let $src: &str = &view;
                let sheet_rc = $crate::css_syntax::sheet_for(Some(node.get_inner_node()), $ctx.source, scss, $crate::css_syntax::is_module($ctx));
                let $sheet: &$crate::css_syntax::Sheet = &sheet_rc;
                let from = $out.len();
                // A closure, so a rule's early `return` still reaches the check below.
                #[allow(clippy::redundant_closure_call)]
                (|| $body)();
                if scss {
                    $crate::css_syntax::keep_line_comments($ctx.source, $out, from);
                }
            }
        }
    };
}
pub(crate) use css_rule;

pub fn finding(start: usize, end: usize, message: impl Into<String>) -> crate::NativeFinding {
    crate::NativeFinding { start, end, message: message.into(), edits: Vec::new() }
}

pub fn fixed(start: usize, end: usize, message: impl Into<String>, edits: Vec<codelaw_core::TextEdit>) -> crate::NativeFinding {
    crate::NativeFinding { start, end, message: message.into(), edits }
}

/// A plain property name: no preprocessor or template syntax, no IE hack.
pub fn standard_property(p: &str) -> bool {
    let b = p.as_bytes();
    b.first().is_some_and(|&c| c.is_ascii_alphabetic() || c == b'-' || c >= 0x80) && b.iter().all(|&c| is_ident_byte(c) || c == b'\\')
}

/// Whether a value uses preprocessor or template syntax.
pub fn non_standard_value(v: &str) -> bool {
    v.contains("#{") || v.contains("${") || v.contains("@{") || v.contains("{{") || v.contains("<%") || v.starts_with('$') || v.contains(" $")
}

/// An option holding a string, or the default.
pub fn opt_str<'a>(ctx: &'a crate::NativeContext<'_>, key: &str, default: &'a str) -> &'a str {
    ctx.options.get(key).and_then(|v| v.as_str()).unwrap_or(default)
}

/// An option holding a list of strings.
pub fn opt_list(ctx: &crate::NativeContext<'_>, key: &str) -> Vec<String> {
    ctx.options.get(key).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default()
}

/// Whether `name` matches one of `patterns`: a plain string matches
/// itself, case-insensitively; `/re/` is a regular expression.
pub fn matches_any(patterns: &[String], name: &str) -> bool {
    patterns.iter().any(|p| match p.strip_prefix('/').and_then(|r| r.strip_suffix('/')) {
        Some(re) => regex::Regex::new(re).is_ok_and(|r| r.is_match(name)),
        None => p.eq_ignore_ascii_case(name),
    })
}

/// Lines from a byte offset, 1-based.
pub fn line(src: &str, at: usize) -> usize {
    crate::util::line_of(src, at)
}

// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statements() {
        let src = "@import url(a.css);\na { color: red !important; --x: { b: c }; &:hover { top: 0 } }\n/* c */\n@media (min-width: 1px) { b { x: y } }";
        let s = parse(src);
        let kinds: Vec<Kind> = s.walk().iter().map(|&i| s.nodes[i].kind).collect();
        assert_eq!(kinds, [Kind::AtRule, Kind::Rule, Kind::Decl, Kind::Decl, Kind::Rule, Kind::Decl, Kind::Comment, Kind::AtRule, Kind::Rule, Kind::Decl]);
        let d = &s.nodes[2];
        assert_eq!(text(src, d.params), "red");
        assert_eq!(text(src, d.important.unwrap()), "!important");
        assert_eq!(text(src, s.nodes[3].params), "{ b: c }");
        assert_eq!(text(src, s.nodes[4].name), "&:hover");
        assert_eq!(text(src, s.nodes[7].params), "(min-width: 1px)");
    }

    #[test]
    fn scss_statements() {
        let src = "// c\n$a: 1px !default;\n@mixin m($x: 1) { top: $x }\n.b {\n  font: { family: x; }\n  &-c { @include m; }\n  background: url(//x.png);\n}\n@import \"a\";\n";
        let s = parse_scss(src);
        let kinds: Vec<Kind> = s.walk().iter().map(|&i| s.nodes[i].kind).collect();
        // `$a` decl, `@mixin` (other) + its decl, `.b` rule, `font:` (other, its block unread), `&-c` rule, `@include` (other), url decl, `@import` (other).
        assert_eq!(kinds, [Kind::Decl, Kind::Other, Kind::Decl, Kind::Rule, Kind::Other, Kind::Rule, Kind::Other, Kind::Decl, Kind::Other]);
        assert!(s.scss);
        assert_eq!(text(src, s.nodes[0].name), "$a");
        assert_eq!(text(src, s.nodes[0].important.unwrap()), "!default");
        // The `//` inside `url()` is not a comment, the one on line 1 is.
        assert_eq!(scss_line_comments(src), [(0, 4)]);
        assert_eq!(scss_view(src).len(), src.len());
    }

    #[test]
    fn interpolation_is_one_unit() {
        let src = "#{$sel} { #{$p}-top: 1px; width: calc(100% - #{$a}); }";
        let s = parse_scss(src);
        assert_eq!(s.nodes.len(), 3);
        assert_eq!(text(src, s.nodes[0].name), "#{$sel}");
        assert_eq!(text(src, s.nodes[1].name), "#{$p}-top");
    }

    #[test]
    fn selectors() {
        let src = "a.b > #c:not(.d, e)::before, [x=\"y\"] ~ * + svg|rect";
        let l = selector_list(src, 0, src.len());
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].compounds.len(), 2);
        assert_eq!(l[0].combinators, [">"]);
        assert_eq!(l[1].combinators, ["~", "+"]);
        assert_eq!(specificity(src, &l[0], (0, 0, 0)), (1, 2, 2));
        assert_eq!(normalize_selector("a  >  b ,c"), "a>b,c");
    }

    #[test]
    fn value_nodes() {
        let src = "rgb(0 0 0 / 50%), url(a b.png) 1.5e3px";
        let v = values(src, 0, src.len());
        let sig = significant(&v);
        assert_eq!(sig.len(), 4);
        assert_eq!(sig[0].name(src), "rgb");
        assert_eq!(sig[2].nodes[0].text(src), "a b.png");
        assert_eq!(dimension("1.5e3px"), Some(("1.5e3", "px")));
        assert_eq!(dimension("1em"), Some(("1", "em")));
        assert_eq!(dimension("auto"), None);
    }
}
