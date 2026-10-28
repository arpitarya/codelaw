//! The statements of a stylesheet, read from its tree-sitter tree (W-62;
//! SR-ARCHITECTURE decision 15b). The CSS fork and the SCSS fork built on it
//! (decision 3) read modern CSS and Sass, so the `css/*` and `scss/*` rules
//! take their rules, at-rules, declarations and comments from the tree: which
//! node is a statement, where its block opens and closes, where its colon,
//! its `!` annotation and its `;` are. The ranges a rule reads are the ones
//! [`css_syntax`](crate::css_syntax) has always given (a selector trimmed up
//! to its `{`, a value without its annotation), so a rule's findings do not
//! depend on which reader built its sheet.
//!
//! A tree with an `ERROR` or a `MISSING` node is not read: the file's
//! statements come from the text reader instead, which never fails. That is
//! the CSS the grammar rejects (W-62's *Left* item 2 lists the forms). So is
//! a CSS tree holding a `//` comment, which the grammar reads and CSS does not
//! (`css/no-invalid-double-slash-comments` reports the statement it breaks).

use crate::css_syntax::{Kind, Node, SASS_AT_RULES, Sheet, ident_end, non_standard, trim};
use tree_sitter::{Node as TsNode, TreeCursor};

/// The sheet of a tree, or `None` when the tree holds an error.
pub fn sheet(root: TsNode<'_>, src: &str, scss: bool, module: bool) -> Option<Sheet> {
    let kinds = kind_names(root)?;
    if root.has_error() || (!scss && src.contains("//") && !crate::css_syntax::scss_line_comments(src).is_empty()) {
        return None;
    }
    let mut b = Builder { src, nodes: Vec::new(), scss, module, w: Walk { cur: root.walk(), kinds } };
    let top = b.statements(root, None);
    Some(Sheet { nodes: b.nodes, top, scss, module })
}

struct Builder<'s, 't> {
    src: &'s str,
    nodes: Vec<Node>,
    scss: bool,
    module: bool,
    w: Walk<'t>,
}

/// A grammar's node kind names by id. `Node::kind` measures and checks its C
/// string on every call; the walk asks for kinds often enough that a table
/// lookup by `kind_id` is most of the difference in speed.
type Kinds = &'static [String];

/// One cursor for the whole walk (a cursor per node costs more than the rest
/// of the reading) and the grammar's kind names.
struct Walk<'t> {
    cur: TreeCursor<'t>,
    kinds: Kinds,
}

/// The kind names of the tree's grammar: the CSS fork's or the SCSS fork's,
/// read once each.
fn kind_names(root: TsNode<'_>) -> Option<Kinds> {
    static TABLES: [std::sync::OnceLock<Vec<String>>; 2] = [std::sync::OnceLock::new(), std::sync::OnceLock::new()];
    let lang = root.language();
    let count = lang.node_kind_count();
    let names = || (0..count).map(|i| u16::try_from(i).ok().and_then(|i| lang.node_kind_for_id(i)).unwrap_or("").to_owned()).collect::<Vec<_>>();
    for t in &TABLES {
        let table = t.get_or_init(names);
        let last = count.checked_sub(1).and_then(|i| u16::try_from(i).ok()).and_then(|i| lang.node_kind_for_id(i));
        if table.len() == count && table.last().map(String::as_str) == last {
            return Some(table);
        }
    }
    None
}

fn kd(kinds: Kinds, n: &TsNode<'_>) -> &'static str {
    kinds.get(usize::from(n.kind_id())).map_or("", String::as_str)
}

fn children<'t>(n: TsNode<'t>, cur: &mut TreeCursor<'t>) -> Vec<TsNode<'t>> {
    n.children(cur).collect()
}

/// The `;` that ends a statement, as its last child.
fn semicolon_of(kinds: Kinds, n: TsNode<'_>) -> Option<usize> {
    let last = n.child(n.child_count().checked_sub(1)?)?;
    (kd(kinds, &last) == ";").then(|| last.start_byte())
}

/// Whether a rule prelude opens an ICSS block (`:export`, `:import("x")`).
fn icss_block(prelude: &str) -> bool {
    let t = prelude.trim();
    t == ":export" || t.starts_with(":import(") || t.starts_with(":import (")
}

impl<'t> Builder<'_, 't> {
    fn push(&mut self, n: Node) -> usize {
        self.nodes.push(n);
        self.nodes.len() - 1
    }

    fn node(kind: Kind, start: usize, end: usize, parent: Option<usize>) -> Node {
        Node { kind, start, end, parent, children: Vec::new(), name: (start, start), params: (start, start), important: None, block: None, semicolon: None, selectors: None }
    }

    /// The statements directly inside `container` (the stylesheet or a block).
    fn statements(&mut self, container: TsNode<'t>, parent: Option<usize>) -> Vec<usize> {
        let mut list = Vec::new();
        for c in children(container, &mut self.w.cur) {
            self.statement(c, parent, &mut list);
        }
        list
    }

    fn statement(&mut self, c: TsNode<'t>, parent: Option<usize>, list: &mut Vec<usize>) {
        let kinds = self.w.kinds;
        let src = self.src;
        let (start, end) = (c.start_byte(), c.end_byte());
        match kd(kinds, &c) {
            "{" | "}" | ";" | "js_comment" => {}
            "comment" => {
                let mut n = Self::node(Kind::Comment, start, end, parent);
                n.params = (start + 2, if src[start..end].ends_with("*/") && end - start >= 4 { end - 2 } else { end });
                list.push(self.push(n));
            }
            "rule_set" | "keyframe_block" => {
                let kids = children(c, &mut self.w.cur);
                let block = kids.iter().rev().copied().find(|k| matches!(kd(kinds, k), "block" | "keyframe_block_list"));
                let open = block.map_or(end, |b| b.start_byte());
                let icss = self.module && icss_block(&src[start..open]);
                let mut n = Self::node(if icss { Kind::Other } else { Kind::Rule }, start, end, parent);
                n.name = trim(src, start, open);
                // Interpolated or templated selectors (`.a-#{$x}`) the rules
                // skip; the tree models them in a way the selector model has no
                // kind for, so they keep their text reading.
                if kd(kinds, &c) == "rule_set" && !non_standard(&src[n.name.0..n.name.1]) {
                    n.selectors = kids.iter().copied().find(|k| kd(kinds, k) == "selectors").map(|k| selectors(k, src, &mut self.w));
                }
                let id = self.push(n);
                list.push(id);
                self.finish(id, block, !icss);
            }
            "declaration" => self.declaration(c, parent, list),
            _ if src[start..].starts_with('@') => self.at_rule(c, parent, list),
            _ => {
                // Nothing a clean tree puts here; kept, as the text reader keeps
                // what it cannot place.
                let mut n = Self::node(Kind::Other, start, end, parent);
                n.semicolon = semicolon_of(kinds, c);
                n.end = trim(src, start, n.semicolon.unwrap_or(end)).1;
                list.push(self.push(n));
            }
        }
    }

    /// An at-rule: CSS's own (`@media`, `@font-face`, any `at_rule`) or Sass's
    /// (`@include`, `@if`, …, kept as `Other`). An `@if`'s `@else` clauses are
    /// statements of their own, as the text reads them.
    fn at_rule(&mut self, c: TsNode<'t>, parent: Option<usize>, list: &mut Vec<usize>) {
        let kinds = self.w.kinds;
        let src = self.src;
        let start = c.start_byte();
        let name_end = ident_end(src, start + 1);
        let lower = src[start + 1..name_end].to_ascii_lowercase();
        let sass = (self.scss && SASS_AT_RULES.contains(&lower.as_str())) || (self.module && lower == "value");
        let kids = children(c, &mut self.w.cur);
        let block_at = kids.iter().position(|k| matches!(kd(kinds, k), "block" | "keyframe_block_list"));
        let block = block_at.map(|i| kids[i]);
        let mut end = c.end_byte();
        if let Some(b) = block {
            end = b.end_byte();
        }
        let mut n = Self::node(if sass { Kind::Other } else { Kind::AtRule }, start, end, parent);
        n.name = (start + 1, name_end);
        let stop = block.map_or_else(|| semicolon_of(kinds, c).unwrap_or(end), |b| b.start_byte());
        n.params = trim(src, name_end, stop);
        if block.is_none() {
            n.semicolon = semicolon_of(kinds, c);
            n.end = trim(src, start, n.semicolon.unwrap_or(end)).1;
        }
        let id = self.push(n);
        list.push(id);
        if block.is_some() {
            self.finish(id, block, true);
        }
        // `@else` / `@else if` clauses after the block.
        if let Some(i) = block_at {
            for k in &kids[i + 1..] {
                if kd(kinds, k).starts_with("else") {
                    self.at_rule(*k, parent, list);
                }
            }
        }
    }

    fn declaration(&mut self, c: TsNode<'t>, parent: Option<usize>, list: &mut Vec<usize>) {
        let kinds = self.w.kinds;
        let src = self.src;
        let start = c.start_byte();
        // The children are read by position, not collected: a declaration is
        // `name : value… [!annotation] [;]` (or `name : { … }`).
        let count = c.child_count();
        let colon = match c.child(1).filter(|k| kd(kinds, k) == ":") {
            Some(k) => k.start_byte(),
            None => children(c, &mut self.w.cur).into_iter().find(|k| kd(kinds, k) == ":").map_or(start, |k| k.start_byte()),
        };
        let block = c.child(2).filter(|k| kd(kinds, k) == "block");
        let custom = src[start..].starts_with("--");
        if let Some(b) = block
            && !custom
        {
            // A Sass nested property (`font: { family: x }`): its statements
            // are property suffixes, not declarations, and are not kept.
            let mut n = Self::node(Kind::Other, start, b.end_byte(), parent);
            n.name = trim(src, start, b.start_byte());
            n.block = Some((b.start_byte(), b.end_byte() - 1));
            list.push(self.push(n));
            return;
        }
        let last = count.checked_sub(1).and_then(|i| c.child(i));
        let semicolon = last.filter(|k| kd(kinds, k) == ";").map(|k| k.start_byte());
        let stop = semicolon.unwrap_or(c.end_byte());
        let mut n = Self::node(Kind::Decl, start, stop, parent);
        n.name = trim(src, start, colon);
        if self.module && src[n.name.0..n.name.1].eq_ignore_ascii_case("composes") {
            n.kind = Kind::Other;
        }
        let (vs, ve) = trim(src, colon + 1, stop);
        n.params = (vs, ve);
        if !custom && src[vs..ve].contains('!') {
            let annotation = |k: &TsNode<'_>| matches!(kd(kinds, k), "important" | "important_value");
            let before = count.checked_sub(if semicolon.is_some() { 2 } else { 1 }).and_then(|i| c.child(i)).filter(annotation);
            let bang = before.or_else(|| children(c, &mut self.w.cur).into_iter().rev().find(annotation));
            if let Some(bang) = bang {
                n.important = Some((bang.start_byte(), ve));
                n.params = trim(src, vs, bang.start_byte());
            }
        }
        n.end = trim(src, start, stop).1;
        n.semicolon = semicolon;
        list.push(self.push(n));
    }

    /// Give a statement its block and, when `keep`, the statements in it.
    fn finish(&mut self, id: usize, block: Option<TsNode<'t>>, keep: bool) {
        let Some(b) = block else { return };
        let kids = if keep { self.statements(b, Some(id)) } else { Vec::new() };
        let close = if self.src[..b.end_byte()].ends_with('}') { b.end_byte() - 1 } else { b.end_byte() };
        let n = &mut self.nodes[id];
        n.children = kids;
        n.block = Some((b.start_byte(), close));
        n.end = b.end_byte();
    }
}

// ---------------------------------------------------------------------
// Selectors

use crate::css_syntax::{Complex, Compound, SKind, Simple};

/// One piece of a selector in source order: a simple selector, or a
/// combinator (`" "` for a descendant).
enum Piece {
    Simple(Simple),
    Comb(String),
}

/// The selector list of a `selectors` node: each complex selector as the
/// text reader's [`selector_list`](crate::css_syntax::selector_list) gives
/// it, from the tree.
fn selectors<'t>(sel: TsNode<'t>, src: &str, w: &mut Walk<'t>) -> Vec<Complex> {
    let kinds = w.kinds;
    children(sel, &mut w.cur).into_iter().filter(|c| c.is_named() && !comment(kinds, *c)).map(|c| complex(c, src, w)).collect()
}

fn complex<'t>(n: TsNode<'t>, src: &str, w: &mut Walk<'t>) -> Complex {
    let mut pieces = Vec::new();
    pieces_of(n, src, &mut pieces, w);
    let mut out = Complex { start: n.start_byte(), end: n.end_byte(), compounds: Vec::new(), combinators: Vec::new(), leading: None };
    let mut pending: Option<String> = None;
    let mut cur: Option<Compound> = None;
    for p in pieces {
        match p {
            Piece::Comb(c) => {
                if let Some(comp) = cur.take() {
                    out.compounds.push(comp);
                }
                pending = Some(c);
            }
            Piece::Simple(s) => {
                if cur.is_none() {
                    if out.compounds.is_empty() {
                        out.leading = pending.take();
                    } else {
                        out.combinators.push(pending.take().unwrap_or_else(|| " ".to_owned()));
                    }
                }
                let comp = cur.get_or_insert(Compound { start: s.start, end: s.end, parts: Vec::new() });
                comp.end = s.end;
                comp.parts.push(s);
            }
        }
    }
    if let Some(comp) = cur {
        out.compounds.push(comp);
    }
    out
}

fn comment(kinds: Kinds, n: TsNode<'_>) -> bool {
    matches!(kd(kinds, &n), "comment" | "js_comment")
}

fn simple(kind: SKind, start: usize, end: usize, name: (usize, usize), args: Option<(usize, usize)>) -> Piece {
    Piece::Simple(Simple { kind, start, end, name, args })
}

/// The inside of an `arguments` node's parentheses.
fn args_of<'t>(kinds: Kinds, kids: &[TsNode<'t>]) -> Option<(usize, usize)> {
    let a = kids.iter().copied().find(|c| kd(kinds, c) == "arguments")?;
    Some((a.start_byte() + 1, a.end_byte().saturating_sub(1).max(a.start_byte() + 1)))
}

fn pieces_of<'t>(n: TsNode<'t>, src: &str, out: &mut Vec<Piece>, w: &mut Walk<'t>) {
    let kinds = w.kinds;
    let kids = children(n, &mut w.cur);
    let (start, end) = (n.start_byte(), n.end_byte());
    // A compound part's own text starts at its first token after the selector
    // it is joined to (`.b` in `a.b`).
    let left = |out: &mut Vec<Piece>, w: &mut Walk<'t>| -> usize {
        match kids.first() {
            Some(f) if f.is_named() && !comment(kinds, *f) && f.start_byte() == start && kids.len() > 1 && !matches!(kd(kinds, &n), "namespace_selector") => {
                pieces_of(*f, src, out, w);
                kids[1].start_byte()
            }
            _ => start,
        }
    };
    match kd(kinds, &n) {
        "descendant_selector" => {
            let named: Vec<_> = kids.iter().filter(|k| !comment(kinds, **k)).collect();
            pieces_of(*named[0], src, out, w);
            out.push(Piece::Comb(" ".to_owned()));
            pieces_of(*named[named.len() - 1], src, out, w);
        }
        "child_selector" | "sibling_selector" | "adjacent_sibling_selector" => {
            for k in kids.iter().filter(|k| !comment(kinds, **k)) {
                if k.is_named() {
                    pieces_of(*k, src, out, w);
                } else {
                    out.push(Piece::Comb(kd(kinds, k).to_owned()));
                }
            }
        }
        "class_selector" if !kids.iter().any(|k| kd(kinds, k) == ".") => {
            // Sass's `&-suffix`: the nesting selector and its suffix are one.
            let at = kids.iter().position(|k| kd(kinds, k) == "nesting_selector").unwrap_or(0);
            for k in &kids[..at] {
                pieces_of(*k, src, out, w);
            }
            let s = kids.get(at).map_or(start, |k| k.start_byte());
            out.push(simple(SKind::Nesting, s, end, (s, end), None));
        }
        "class_selector" => {
            let s = left(out, w);
            let name = kids.iter().find(|k| k.start_byte() > s && kd(kinds, k) == "class_name").map_or((s, end), |k| (k.start_byte(), k.end_byte()));
            out.push(simple(SKind::Class, s, end, name, None));
        }
        "id_selector" => {
            let s = left(out, w);
            let name = kids.iter().find(|k| k.start_byte() > s && kd(kinds, k) == "id_name").map_or((s + 1, end), |k| (k.start_byte(), k.end_byte()));
            out.push(simple(SKind::Id, s, end, name, None));
        }
        "pseudo_class_selector" | "pseudo_element_selector" => {
            let s = left(out, w);
            let element = kd(kinds, &n) == "pseudo_element_selector";
            let name = kids.iter().find(|k| k.start_byte() > s && matches!(kd(kinds, k), "class_name" | "tag_name")).map_or((s, s), |k| (k.start_byte(), k.end_byte()));
            out.push(simple(if element { SKind::PseudoElement } else { SKind::PseudoClass }, s, end, name, args_of(kinds, &kids)));
        }
        "attribute_selector" => {
            let s = left(out, w);
            let inner = crate::css_syntax::trim(src, s + 1, end - 1);
            let ne = ident_end(src, inner.0).min(end - 1);
            out.push(simple(SKind::Attribute, s, end, (inner.0, ne), Some(inner)));
        }
        "namespace_selector" => {
            // `ns|a`, `*|a`, `|a`, `ns|*`: one simple selector. The grammar
            // reads `x ns|a` as `(x ns)|a`: the selectors before the prefix are
            // pieces of their own.
            let mut s = start;
            if let Some(f) = kids.first().filter(|f| f.is_named() && !matches!(kd(kinds, f), "tag_name" | "universal_selector")) {
                pieces_of(*f, src, out, w);
                if let Some(Piece::Simple(last)) = out.pop() {
                    s = last.start;
                }
            }
            let star = src[..end].ends_with('*');
            out.push(simple(if star { SKind::Universal } else { SKind::Type }, s, end, (s, end), None));
        }
        "tag_name" => out.push(simple(SKind::Type, start, end, (start, end), None)),
        "universal_selector" => out.push(simple(SKind::Universal, start, end, (start, end), None)),
        "nesting_selector" => {
            // Sass's parent suffix (`&-wide`) is part of the nesting selector.
            let e = ident_end(src, start + 1).max(end);
            out.push(simple(SKind::Nesting, start, e, (start, e), None));
        }
        _ => out.push(simple(SKind::Other, start, end, (start, end), None)),
    }
}

#[cfg(test)]
mod tests {
    use crate::css_syntax::{Sheet, parse_with, rule_source, selector_list};
    use codelaw_core::Language;

    /// Every node as the rules read it.
    fn shape(s: &Sheet, src: &str) -> Vec<String> {
        s.walk()
            .into_iter()
            .map(|i| {
                let n = &s.nodes[i];
                let t = |r: (usize, usize)| &src[r.0..r.1];
                let sel = n.selectors.as_ref().map(|l| format!("{l:?}"));
                format!("{:?} {}..{} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {sel:?}", n.kind, n.start, n.end, t(n.name), t(n.params), n.important.map(t), n.block, n.semicolon, n.children, n.parent)
            })
            .collect()
    }

    /// The tree's sheet, which must read like the text's.
    fn agree(language: Language, src: &str, module: bool) {
        let scss = language == Language::Scss;
        let root = codelaw_lang::parse(language, src);
        let view = rule_source(src, scss);
        let tree = super::sheet(root.root().get_inner_node(), &view, scss, module).expect("a clean tree");
        let text = parse_with(src, scss, module);
        let mut a = shape(&text, &view);
        let b = shape(&tree, &view);
        // The text reader leaves `selectors` empty: compare the tree's with a
        // text reading of the same selector.
        for (i, n) in tree.nodes.iter().enumerate() {
            if let Some(l) = &n.selectors {
                assert_eq!(format!("{l:?}"), format!("{:?}", selector_list(&view, n.name.0, n.name.1)));
                let k = tree.walk().iter().position(|&w| w == i).unwrap();
                a[k] = b[k].clone();
            }
        }
        assert_eq!(a, b);
    }

    #[test]
    fn the_tree_reads_like_the_text() {
        agree(
            Language::Css,
            "@charset \"utf-8\";\n@import url(\"x.css\") layer(a) supports(display: grid) screen;\n.a, .b > c:hover { color: red !important; /* c */ margin: 0 auto }\n@media (min-width: 1px) { .x { a: b } }\n@font-face { font-family: x }\n@keyframes k { from { a: b } 50% { a: c } }\n:root { --v: { a: b }; --w: 1 !important }\n@unknown foo bar;\n.n { .m & { a: b } &:hover { c: d } > .e { f: g } }\nns|a *|b, a[href=\"x\"]::before, li:nth-child(2n of .a):not(.b, #c) { x: y }\n",
            false,
        );
        agree(
            Language::Scss,
            "@use \"sass:map\";\n$x: 1px !default;\n// note\n.p { &:not(.q) { &::after { x: y } } font: { family: z; } @include m(1) { top: 0; } &-wide { a: b } }\n@mixin m($a) { b: $a; }\n@if $a { a { b: c } } @else if $b { d { e: f } } @else { g { h: i } }\n.u { margin: -$x; width: calc(100% - #{$x}) }\n.a-#{$x} { b: c }\n",
            false,
        );
        agree(Language::Css, ":export { a: b }\n.a { composes: b; c: d }\n", true);
    }

    #[test]
    fn a_tree_with_an_error_or_a_css_line_comment_is_not_read() {
        for src in [".a { color: red; ", "// a\nb { c: d }"] {
            let root = codelaw_lang::parse(Language::Css, src);
            assert!(super::sheet(root.root().get_inner_node(), src, false, false).is_none(), "{src}");
        }
    }
}
