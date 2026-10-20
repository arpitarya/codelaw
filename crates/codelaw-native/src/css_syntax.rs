//! A small CSS reader for the `css/*` native rules. tree-sitter-css drops
//! to ERROR nodes on much of modern CSS (`50.5%` keyframe selectors,
//! `layer()` imports, custom properties holding blocks), so the CSS rules
//! read the stylesheet's text themselves: statements (rules, at-rules,
//! declarations, comments), component values, selectors and media
//! queries, every piece with its byte range in the source.
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
    /// The statement ends with `;`, at this offset.
    pub semicolon: Option<usize>,
}

#[derive(Debug, Default)]
pub struct Sheet {
    pub nodes: Vec<Node>,
    pub top: Vec<usize>,
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
    /// The nearest enclosing at-rule.
    pub fn at_rule_context(&self, i: usize) -> Option<usize> {
        let mut p = self.nodes[i].parent;
        while let Some(x) = p {
            match self.nodes[x].kind {
                Kind::AtRule => return Some(x),
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

fn trim(src: &str, start: usize, end: usize) -> Range {
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
}

impl Parser<'_> {
    fn push(&mut self, n: Node) -> usize {
        self.nodes.push(n);
        self.nodes.len() - 1
    }

    fn node(kind: Kind, start: usize, end: usize, parent: Option<usize>) -> Node {
        Node { kind, start, end, parent, children: Vec::new(), name: (start, start), params: (start, start), important: None, block: None, semicolon: None }
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
                let mut n = Self::node(Kind::AtRule, start, stop_at, parent);
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
                let mut n = Self::node(Kind::Rule, start, stop_at, parent);
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

pub fn parse(src: &str) -> Sheet {
    let mut p = Parser { src, nodes: Vec::new() };
    let (top, _) = p.block(0, None);
    Sheet { nodes: p.nodes, top }
}

thread_local! {
    static CACHE: RefCell<Option<(String, Rc<Sheet>)>> = const { RefCell::new(None) };
}

/// The parsed sheet of a source; each file is parsed once, however many
/// CSS rules run on it.
pub fn sheet(src: &str) -> Rc<Sheet> {
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((s, sheet)) = c.as_ref()
            && s == src
        {
            return sheet.clone();
        }
        let sheet = Rc::new(parse(src));
        *c = Some((src.to_owned(), sheet.clone()));
        sheet
    })
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
                i += 1;
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
                &[codelaw_core::Language::Css]
            }
            fn kinds(&self) -> &'static [&'static str] {
                &["stylesheet"]
            }
            #[allow(unused_variables)]
            fn check(&self, _node: codelaw_lang::SyntaxNode<'_>, $ctx: &$crate::NativeContext<'_>, $out: &mut Vec<$crate::NativeFinding>) {
                let $src: &str = $ctx.source;
                let sheet_rc = $crate::css_syntax::sheet($src);
                let $sheet: &$crate::css_syntax::Sheet = &sheet_rc;
                $body
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
