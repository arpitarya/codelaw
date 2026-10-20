//! The document model behind the `html/*` native rules.
//!
//! tree-sitter-html gives tags and text but knows only a few of HTML's
//! implied end tags. The rules need the tree a browser builds, so this module
//! walks the parse in source order as a stream of tags and text and rebuilds
//! the element tree with a stack, closing elements implicitly where the
//! pack's element table (`html/elements`, from html-validate's metadata)
//! says HTML does. It also records how each element was closed, which the
//! `close-order` and `no-implicit-close` rules report.
//!
//! The model is built once per file and shared by every rule (a one-entry
//! cache per thread, keyed by the source and the table).

use crate::{NativeContext, NativeFinding};
use codelaw_core::TextEdit;
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tree_sitter::Node;

/// An attribute as written.
#[derive(Clone, Debug)]
pub struct Attr {
    /// Lower-cased name.
    pub name: String,
    pub name_start: usize,
    pub name_end: usize,
    /// The value as written (character references not decoded); `None` for
    /// a bare attribute.
    pub value: Option<String>,
    pub value_start: usize,
    pub value_end: usize,
    pub quoted: bool,
    /// The whole attribute, `name="value"`.
    pub start: usize,
    pub end: usize,
}

impl Attr {
    pub fn value_or_empty(&self) -> &str {
        self.value.as_deref().unwrap_or("")
    }
    /// Template code (`{{ }}`, `<% %>`, `${ }`) whose value is only known
    /// at render time.
    pub fn is_dynamic(&self) -> bool {
        self.value.as_deref().is_some_and(is_dynamic)
    }
}

pub fn is_dynamic(s: &str) -> bool {
    s.contains("{{") || s.contains("<%") || s.contains("${") || s.contains("{%")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextKind {
    Plain,
    /// A character reference tree-sitter recognised (`&amp;`, `&#1;`, `&b`).
    Entity,
    /// Script or style content.
    Raw,
    /// Text tree-sitter could not parse: usually a raw `&` or `<`.
    Unparsed,
}

#[derive(Clone, Debug)]
pub struct Text {
    pub start: usize,
    pub end: usize,
    pub kind: TextKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Child {
    Elem(usize),
    Text(usize),
}

/// How an element ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Closed {
    /// Still open (only while building).
    Open,
    EndTag,
    /// A void element, or a `/>` self-closing tag.
    Void,
    /// Its optional end tag was left out; `by` names what closed it.
    Implicit(ClosedBy),
    /// Its end tag is required and missing.
    Unclosed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClosedBy {
    /// The next sibling has the same tag (`<li>a<li>b`).
    Sibling,
    /// A following start tag (`<p>a<div>`).
    Adjacent(String),
    /// The parent's end tag.
    Parent(String),
    /// An ancestor's end tag.
    Ancestor(String),
    EndOfDocument,
}

#[derive(Clone, Debug)]
pub struct Elem {
    /// Lower-cased tag name.
    pub tag: String,
    pub parent: Option<usize>,
    pub children: Vec<Child>,
    pub attrs: Vec<Attr>,
    /// The start tag, `<` to `>`. An implied element (`<head>` opened by a
    /// `<title>`) has an empty range at the tag that implied it.
    pub start: usize,
    pub open_end: usize,
    /// The end tag, when written.
    pub end_tag: Option<(usize, usize)>,
    /// Attribute-like text written inside the end tag (`</p class=x>`).
    pub end_tag_junk: Option<(usize, usize)>,
    /// The end of the element: its end tag, or its last content.
    pub end: usize,
    pub closed: Closed,
    pub implied: bool,
    /// Inside `<svg>` or `<math>`: the HTML content model doesn't apply.
    pub in_foreign: bool,
    pub self_closing_syntax: bool,
}

impl Elem {
    pub fn attr(&self, name: &str) -> Option<&Attr> {
        self.attrs.iter().find(|a| a.name == name)
    }
    pub fn has(&self, name: &str) -> bool {
        self.attr(name).is_some()
    }
    /// The attribute's value, lower-cased and trimmed.
    pub fn value_lower(&self, name: &str) -> Option<String> {
        self.attr(name).and_then(|a| a.value.as_ref()).map(|v| v.trim().to_ascii_lowercase())
    }
    pub fn is(&self, tag: &str) -> bool {
        self.tag == tag
    }
}

#[derive(Clone, Debug)]
pub struct EndTagSeen {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

/// A parsed document.
pub struct Dom {
    pub elems: Vec<Elem>,
    pub texts: Vec<Text>,
    /// Children of the document itself.
    pub roots: Vec<Child>,
    pub comments: Vec<(usize, usize)>,
    pub doctype: Option<(usize, usize)>,
    /// End tags that matched no open element.
    pub stray_end_tags: Vec<EndTagSeen>,
    /// End tags written for void elements (`</br>`).
    pub void_end_tags: Vec<EndTagSeen>,
    /// `<script … />`: a script start tag with `/>`.
    pub self_closed_scripts: Vec<(usize, usize)>,
    pub source: Rc<str>,
}

// ---------------------------------------------------------------- metadata

/// The pack's element table: `{ global: {attributes}, elements: {tag: meta} }`.
#[derive(Clone, Copy)]
pub struct Meta<'d> {
    pub table: Option<&'d Value>,
}

impl<'d> Meta<'d> {
    pub fn from_ctx(ctx: &'d NativeContext<'_>) -> Meta<'d> {
        Meta { table: ctx.data.get("html/elements").map(|v| v.as_ref()) }
    }
    pub fn known(&self) -> bool {
        self.table.is_some()
    }
    pub fn elem(&self, tag: &str) -> Option<&'d Map<String, Value>> {
        self.table?.get("elements")?.get(tag)?.as_object()
    }
    pub fn get(&self, tag: &str, key: &str) -> Option<&'d Value> {
        self.elem(tag)?.get(key)
    }
    /// A property that is literally `true` (not a condition).
    pub fn static_flag(&self, tag: &str, key: &str) -> bool {
        matches!(self.get(tag, key), Some(Value::Bool(true)))
    }
    pub fn is_void(&self, tag: &str) -> bool {
        self.static_flag(tag, "void")
    }
    pub fn list(&self, tag: &str, key: &str) -> Vec<&'d str> {
        self.get(tag, key).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default()
    }
    /// An attribute's metadata on an element, falling back to the global
    /// attributes.
    pub fn attr(&self, tag: &str, name: &str) -> Option<&'d Map<String, Value>> {
        let own = self.elem(tag).and_then(|e| e.get("attributes")).and_then(|a| a.get(name)).and_then(|v| v.as_object());
        own.or_else(|| self.table?.get("global")?.get("attributes")?.get(name)?.as_object())
    }
    /// Whether a static category (`@flow`, `@meta`, …) applies to a tag, as
    /// the parser uses it before the element exists.
    pub fn tag_in_category(&self, tag: &str, category: &str) -> bool {
        match category {
            "@meta" => self.static_flag(tag, "metadata"),
            "@flow" => self.static_flag(tag, "flow"),
            "@flow-not-meta" => self.static_flag(tag, "flow") && !self.static_flag(tag, "metadata"),
            "@sectioning" => self.static_flag(tag, "sectioning"),
            "@heading" => self.static_flag(tag, "heading"),
            "@phrasing" => self.static_flag(tag, "phrasing"),
            "@embedded" => self.static_flag(tag, "embedded"),
            "@interactive" => self.static_flag(tag, "interactive"),
            _ => false,
        }
    }
    /// Whether an open `open` element is closed by a start tag `incoming`.
    pub fn closes(&self, open: &str, incoming: &str) -> bool {
        self.list(open, "implicitClosed").iter().any(|e| if e.starts_with('@') { self.tag_in_category(incoming, e) } else { *e == incoming })
    }
    /// Whether an element's end tag may be left out.
    pub fn can_omit_end(&self, tag: &str) -> bool {
        self.list(tag, "implicitClosed").contains(&tag) || self.static_flag(tag, "optionalEnd")
    }
    pub fn has_implicit_close(&self, tag: &str) -> bool {
        self.get(tag, "implicitClosed").is_some() || self.static_flag(tag, "optionalEnd")
    }
}

// ---------------------------------------------------------------- building

enum Tok<'t> {
    Start { node: Node<'t>, self_closing: bool },
    End { name: String, start: usize, end: usize, junk: Option<(usize, usize)> },
    Text(Text),
    Comment(usize, usize),
    Doctype(usize, usize),
}

fn child_named<'t>(n: Node<'t>, kind: &str) -> Option<Node<'t>> {
    let mut c = n.walk();
    n.children(&mut c).find(|x| x.kind() == kind)
}

fn tokens<'t>(n: Node<'t>, src: &str, out: &mut Vec<Tok<'t>>) {
    match n.kind() {
        "start_tag" => out.push(Tok::Start { node: n, self_closing: false }),
        "self_closing_tag" => out.push(Tok::Start { node: n, self_closing: true }),
        "end_tag" | "erroneous_end_tag" => {
            let name_node = child_named(n, "tag_name").or_else(|| child_named(n, "erroneous_end_tag_name"));
            let name = name_node.map(|t| src[t.byte_range()].to_ascii_lowercase()).unwrap_or_default();
            let junk = child_named(n, "ERROR").map(|e| (e.start_byte(), e.end_byte()));
            out.push(Tok::End { name, start: n.start_byte(), end: n.end_byte(), junk });
        }
        "text" => out.push(Tok::Text(Text { start: n.start_byte(), end: n.end_byte(), kind: TextKind::Plain })),
        "entity" => out.push(Tok::Text(Text { start: n.start_byte(), end: n.end_byte(), kind: TextKind::Entity })),
        "raw_text" => out.push(Tok::Text(Text { start: n.start_byte(), end: n.end_byte(), kind: TextKind::Raw })),
        "comment" => out.push(Tok::Comment(n.start_byte(), n.end_byte())),
        "doctype" => out.push(Tok::Doctype(n.start_byte(), n.end_byte())),
        "ERROR" => {
            // A tag tree-sitter couldn't finish, such as `<script src=x />`.
            if child_named(n, "tag_name").is_some() && src[n.byte_range()].starts_with('<') {
                let self_closing = src[n.byte_range()].trim_end().ends_with("/>");
                out.push(Tok::Start { node: n, self_closing });
                return;
            }
            let t = &src[n.byte_range()];
            // A doctype tree-sitter can't parse, such as `<!DOCTYPE>`.
            if t.as_bytes().get(..9).is_some_and(|b| b.eq_ignore_ascii_case(b"<!doctype")) {
                out.push(Tok::Doctype(n.start_byte(), n.end_byte()));
                return;
            }
            // An end tag that matches nothing open, at the top level.
            if let Some(rest) = t.strip_prefix("</")
                && let Some(name) = rest.strip_suffix('>')
                && !name.is_empty()
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            {
                out.push(Tok::End { name: name.to_ascii_lowercase(), start: n.start_byte(), end: n.end_byte(), junk: None });
                return;
            }
            let mut c = n.walk();
            let kids: Vec<Node> = n.children(&mut c).collect();
            if kids.iter().any(|k| k.kind() != "ERROR" && k.is_named()) {
                for k in kids {
                    tokens(k, src, out);
                }
            } else {
                out.push(Tok::Text(Text { start: n.start_byte(), end: n.end_byte(), kind: TextKind::Unparsed }));
            }
        }
        _ => {
            let mut c = n.walk();
            for k in n.children(&mut c) {
                tokens(k, src, out);
            }
        }
    }
}

fn read_attrs(tag: Node<'_>, src: &str) -> Vec<Attr> {
    let mut out: Vec<Attr> = Vec::new();
    let mut c = tag.walk();
    for a in tag.children(&mut c) {
        if a.kind() == "ERROR" {
            // tree-sitter ends an unquoted value at `=` or `&`; the rest of
            // the value follows as an error.
            if let Some(last) = out.last_mut()
                && !last.quoted
                && last.value.is_some()
                && last.end == a.start_byte()
            {
                last.value_end = a.end_byte();
                last.end = a.end_byte();
                last.value = Some(src[last.value_start..last.value_end].to_owned());
            }
            continue;
        }
        if a.kind() != "attribute" {
            continue;
        }
        let Some(name) = child_named(a, "attribute_name") else { continue };
        let mut attr = Attr {
            name: src[name.byte_range()].to_ascii_lowercase(),
            name_start: name.start_byte(),
            name_end: name.end_byte(),
            value: None,
            value_start: name.end_byte(),
            value_end: name.end_byte(),
            quoted: false,
            start: a.start_byte(),
            end: a.end_byte(),
        };
        if let Some(v) = child_named(a, "attribute_value") {
            attr.value = Some(src[v.byte_range()].to_owned());
            attr.value_start = v.start_byte();
            attr.value_end = v.end_byte();
        } else if let Some(q) = child_named(a, "quoted_attribute_value") {
            attr.quoted = true;
            match child_named(q, "attribute_value") {
                Some(v) => {
                    attr.value = Some(src[v.byte_range()].to_owned());
                    attr.value_start = v.start_byte();
                    attr.value_end = v.end_byte();
                }
                None => {
                    attr.value = Some(String::new());
                    attr.value_start = q.start_byte() + 1;
                    attr.value_end = q.start_byte() + 1;
                }
            }
        }
        out.push(attr);
    }
    out
}

struct Builder<'m> {
    meta: Meta<'m>,
    elems: Vec<Elem>,
    texts: Vec<Text>,
    roots: Vec<Child>,
    stack: Vec<usize>,
    dom_extra: Extra,
}

#[derive(Default)]
struct Extra {
    comments: Vec<(usize, usize)>,
    doctype: Option<(usize, usize)>,
    stray: Vec<EndTagSeen>,
    void_end: Vec<EndTagSeen>,
    self_closed_scripts: Vec<(usize, usize)>,
}

impl Builder<'_> {
    fn top(&self) -> Option<usize> {
        self.stack.last().copied()
    }
    fn append(&mut self, child: Child) {
        match self.top() {
            Some(p) => self.elems[p].children.push(child),
            None => self.roots.push(child),
        }
    }
    fn new_elem(&mut self, tag: String, start: usize, open_end: usize, attrs: Vec<Attr>, implied: bool) -> usize {
        let parent = self.top();
        let in_foreign = parent.is_some_and(|p| self.elems[p].in_foreign || self.meta.static_flag(&self.elems[p].tag, "foreign"));
        let idx = self.elems.len();
        self.elems.push(Elem {
            tag,
            parent,
            children: Vec::new(),
            attrs,
            start,
            open_end,
            end_tag: None,
            end_tag_junk: None,
            end: open_end,
            closed: Closed::Open,
            implied,
            in_foreign,
            self_closing_syntax: false,
        });
        self.append(Child::Elem(idx));
        idx
    }
    fn start(&mut self, node: Node<'_>, self_closing: bool, src: &str) {
        let Some(name) = child_named(node, "tag_name") else { return };
        let tag = src[name.byte_range()].to_ascii_lowercase();
        let in_foreign = self.top().is_some_and(|p| self.elems[p].in_foreign || self.meta.static_flag(&self.elems[p].tag, "foreign"));
        if !in_foreign {
            while let Some(top) = self.top() {
                if self.meta.closes(&self.elems[top].tag, &tag) {
                    let by = if self.elems[top].tag == tag { ClosedBy::Sibling } else { ClosedBy::Adjacent(tag.clone()) };
                    self.elems[top].closed = Closed::Implicit(by);
                    self.stack.pop();
                } else {
                    break;
                }
            }
            if let Some(parent) = self.top()
                && let Some(Value::Array(rules)) = self.meta.get(&self.elems[parent].tag, "implicitOpen")
            {
                let open = rules.iter().find_map(|r| {
                    let matches = r.get("for")?.as_array()?.iter().filter_map(|s| s.as_str()).any(|s| {
                        if s.starts_with('@') { self.meta.tag_in_category(&tag, s) } else { s == tag }
                    });
                    if matches { r.get("open")?.as_str() } else { None }
                });
                if let Some(open) = open {
                    let at = node.start_byte();
                    let i = self.new_elem(open.to_owned(), at, at, Vec::new(), true);
                    self.stack.push(i);
                }
            }
        }
        let attrs = read_attrs(node, src);
        let idx = self.new_elem(tag.clone(), node.start_byte(), node.end_byte(), attrs, false);
        self.elems[idx].self_closing_syntax = self_closing;
        if tag == "script" && self_closing {
            self.dom_extra.self_closed_scripts.push((node.start_byte(), node.end_byte()));
        }
        if self.meta.is_void(&tag) || self_closing {
            self.elems[idx].closed = Closed::Void;
        } else {
            self.stack.push(idx);
        }
    }
    fn end(&mut self, name: String, start: usize, end: usize, junk: Option<(usize, usize)>) {
        let seen = EndTagSeen { name: name.clone(), start, end };
        if self.meta.is_void(&name) {
            self.dom_extra.void_end.push(seen);
            return;
        }
        let Some(pos) = self.stack.iter().rposition(|&i| self.elems[i].tag == name) else {
            self.dom_extra.stray.push(seen);
            return;
        };
        let target = self.stack[pos];
        let mut early = false;
        for &i in self.stack[pos + 1..].iter().rev() {
            let tag = self.elems[i].tag.clone();
            self.elems[i].closed = if self.meta.can_omit_end(&tag) && !early {
                Closed::Implicit(if self.elems[i].parent == Some(target) { ClosedBy::Parent(name.clone()) } else { ClosedBy::Ancestor(name.clone()) })
            } else {
                early = true;
                Closed::Unclosed
            };
        }
        self.stack.truncate(pos);
        let e = &mut self.elems[target];
        e.closed = Closed::EndTag;
        e.end_tag = Some((start, end));
        e.end_tag_junk = junk;
    }
    fn finish(&mut self) {
        while let Some(i) = self.stack.pop() {
            let tag = self.elems[i].tag.clone();
            self.elems[i].closed =
                if self.meta.has_implicit_close(&tag) { Closed::Implicit(ClosedBy::EndOfDocument) } else { Closed::Unclosed };
        }
    }
}

fn compute_end(elems: &mut Vec<Elem>, texts: &[Text], i: usize) -> usize {
    let mut end = elems[i].open_end;
    let children = elems[i].children.clone();
    for c in children {
        let e = match c {
            Child::Elem(j) => compute_end(elems, texts, j),
            Child::Text(t) => texts[t].end,
        };
        end = end.max(e);
    }
    if let Some((_, e)) = elems[i].end_tag {
        end = end.max(e);
    }
    elems[i].end = end;
    end
}

impl Dom {
    pub fn build(root: Node<'_>, src: &str, meta: Meta<'_>) -> Dom {
        let mut toks = Vec::new();
        tokens(root, src, &mut toks);
        let mut b = Builder { meta, elems: Vec::new(), texts: Vec::new(), roots: Vec::new(), stack: Vec::new(), dom_extra: Extra::default() };
        for t in toks {
            match t {
                Tok::Start { node, self_closing } => b.start(node, self_closing, src),
                Tok::End { name, start, end, junk } => b.end(name, start, end, junk),
                Tok::Text(t) => {
                    let i = b.texts.len();
                    b.texts.push(t);
                    b.append(Child::Text(i));
                }
                Tok::Comment(s, e) => b.dom_extra.comments.push((s, e)),
                Tok::Doctype(s, e) => {
                    if b.dom_extra.doctype.is_none() {
                        b.dom_extra.doctype = Some((s, e));
                    }
                }
            }
        }
        b.finish();
        let roots: Vec<usize> = b.roots.iter().filter_map(|c| if let Child::Elem(i) = c { Some(*i) } else { None }).collect();
        for r in roots {
            compute_end(&mut b.elems, &b.texts, r);
        }
        let x = b.dom_extra;
        Dom {
            elems: b.elems,
            texts: b.texts,
            roots: b.roots,
            comments: x.comments,
            doctype: x.doctype,
            stray_end_tags: x.stray,
            void_end_tags: x.void_end,
            self_closed_scripts: x.self_closed_scripts,
            source: Rc::from(src),
        }
    }

    /// The model of the file a rule is checking, built once per file.
    pub fn get(node: Node<'_>, ctx: &NativeContext<'_>) -> Rc<Dom> {
        type Key = (usize, usize, u64, usize);
        thread_local! {
            static CACHE: RefCell<Option<(Key, Rc<Dom>)>> = const { RefCell::new(None) };
        }
        let src = ctx.source;
        let table = ctx.data.get("html/elements").map(|a| Arc::as_ptr(a) as usize).unwrap_or(0);
        let key: Key = (src.as_ptr() as usize, src.len(), fnv(src.as_bytes()), table);
        CACHE.with(|c| {
            if let Some((k, dom)) = &*c.borrow()
                && *k == key
            {
                return dom.clone();
            }
            let dom = Rc::new(Dom::build(node, src, Meta::from_ctx(ctx)));
            *c.borrow_mut() = Some((key, dom.clone()));
            dom
        })
    }

    pub fn text_of(&self, t: &Text) -> &str {
        &self.source[t.start..t.end]
    }

    /// Every element, in document order.
    pub fn all(&self) -> impl Iterator<Item = usize> + '_ {
        0..self.elems.len()
    }

    pub fn by_tag<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = usize> + 'a {
        self.all().filter(move |&i| self.elems[i].tag == tag)
    }

    pub fn child_elems(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        self.elems[i].children.iter().filter_map(|c| if let Child::Elem(j) = c { Some(*j) } else { None })
    }

    /// Every descendant element of `i`, in document order.
    pub fn descendants(&self, i: usize) -> Vec<usize> {
        let mut out = Vec::new();
        self.collect_desc(i, &mut out);
        out
    }
    fn collect_desc(&self, i: usize, out: &mut Vec<usize>) {
        for j in self.child_elems(i).collect::<Vec<_>>() {
            out.push(j);
            self.collect_desc(j, out);
        }
    }

    /// `i` and its ancestors, nearest first.
    pub fn self_and_ancestors(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        std::iter::successors(Some(i), move |&j| self.elems[j].parent)
    }

    pub fn ancestors(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        self.self_and_ancestors(i).skip(1)
    }

    /// The nearest of `i` and its ancestors matching a selector.
    pub fn closest(&self, i: usize, selector: &str) -> Option<usize> {
        self.self_and_ancestors(i).find(|&j| self.matches(j, selector))
    }

    /// Whether an element matches a comma-separated list of simple
    /// selectors: `tag`, `[attr]`, `[attr="value"]`, `tag[attr="value"]`.
    pub fn matches(&self, i: usize, selector: &str) -> bool {
        selector.split(',').any(|s| self.matches_chain(i, s.trim()))
    }
    /// `a > b > c`: `c` here, `b` its parent, `a` the grandparent.
    fn matches_chain(&self, i: usize, s: &str) -> bool {
        let mut parts = s.rsplit('>').map(str::trim);
        let Some(last) = parts.next() else { return false };
        if !self.matches_one(i, last) {
            return false;
        }
        let mut cur = i;
        for p in parts {
            match self.elems[cur].parent {
                Some(par) if self.matches_one(par, p) => cur = par,
                _ => return false,
            }
        }
        true
    }
    fn matches_one(&self, i: usize, s: &str) -> bool {
        let e = &self.elems[i];
        let (tag, rest) = match s.find('[') {
            Some(p) => (&s[..p], &s[p..]),
            None => (s, ""),
        };
        if !tag.is_empty() && tag != "*" && !e.tag.eq_ignore_ascii_case(tag) {
            return false;
        }
        let mut rest = rest;
        while let Some(stripped) = rest.strip_prefix('[') {
            let Some(close) = stripped.find(']') else { return false };
            let inner = &stripped[..close];
            rest = &stripped[close + 1..];
            match inner.split_once('=') {
                Some((k, v)) => {
                    let v = v.trim_matches(['"', '\'']);
                    if e.attr(k.trim()).and_then(|a| a.value.as_deref()) != Some(v) {
                        return false;
                    }
                }
                None => {
                    if !e.has(inner.trim()) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Text and entity children of `i` and its descendants.
    pub fn text_content(&self, i: usize) -> String {
        let mut s = String::new();
        self.collect_text(i, &mut s);
        s
    }
    fn collect_text(&self, i: usize, s: &mut String) {
        for c in &self.elems[i].children {
            match c {
                Child::Text(t) => {
                    let t = &self.texts[*t];
                    if t.kind != TextKind::Raw {
                        s.push_str(&self.source[t.start..t.end]);
                    }
                }
                Child::Elem(j) => self.collect_text(*j, s),
            }
        }
    }

    /// Every element whose `id` is `id`.
    pub fn by_id<'a>(&'a self, id: &'a str) -> impl Iterator<Item = usize> + 'a {
        self.all().filter(move |&i| self.elems[i].attr("id").and_then(|a| a.value.as_deref()) == Some(id))
    }

    /// The start tag's range, or the tag that implied the element.
    pub fn tag_range(&self, i: usize) -> (usize, usize) {
        let e = &self.elems[i];
        if e.implied {
            // An implied element is reported at the tag that implied it.
            let next = self.child_elems(i).next().map(|j| self.elems[j].open_end).unwrap_or(e.open_end);
            return (e.start, next.max(e.start));
        }
        (e.start, e.open_end)
    }

    /// A finding on an element's start tag.
    pub fn at_elem(&self, i: usize, message: impl Into<String>) -> NativeFinding {
        let (start, end) = self.tag_range(i);
        NativeFinding { start, end, message: message.into(), edits: Vec::new() }
    }
}

pub fn finding(start: usize, end: usize, message: impl Into<String>) -> NativeFinding {
    NativeFinding { start, end, message: message.into(), edits: Vec::new() }
}

pub fn finding_with(start: usize, end: usize, message: impl Into<String>, edits: Vec<TextEdit>) -> NativeFinding {
    NativeFinding { start, end, message: message.into(), edits }
}

/// The edit that removes an attribute together with the whitespace before it.
pub fn remove_attr(src: &str, a: &Attr) -> TextEdit {
    let before = src[..a.start].trim_end_matches([' ', '\t', '\n', '\r']).len();
    TextEdit::new(before, a.end, "")
}

fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

// ---------------------------------------------------------------- conditions

/// Evaluate a condition from the element table against element `i`.
pub fn eval(dom: &Dom, i: usize, cond: &Value) -> bool {
    let e = &dom.elems[i];
    match cond {
        Value::Bool(b) => *b,
        Value::Object(o) => {
            if let Some(v) = o.get("attr") {
                return v.as_str().is_some_and(|n| e.has(n));
            }
            if let Some(v) = o.get("attr-nonempty") {
                return v.as_str().is_some_and(|n| e.attr(n).is_some_and(|a| !a.value_or_empty().is_empty()));
            }
            if let Some(v) = o.get("attr-value") {
                let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
                let values: Vec<&str> = v.get("values").and_then(|x| x.as_array()).map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
                let actual = match e.attr(name) {
                    Some(a) => Some(a.value_or_empty().to_ascii_lowercase()),
                    None => v.get("default").and_then(|d| d.as_str()).map(str::to_owned),
                };
                return actual.is_some_and(|a| values.contains(&a.as_str()));
            }
            if let Some(v) = o.get("attr-token") {
                let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
                let values: Vec<&str> = v.get("values").and_then(|x| x.as_array()).map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
                return e.value_lower(name).is_some_and(|val| val.split_ascii_whitespace().any(|t| values.contains(&t)));
            }
            if let Some(v) = o.get("attr-int-above") {
                let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
                let limit = v.get("value").and_then(|x| x.as_i64()).unwrap_or(0);
                return e.value_lower(name).and_then(|s| s.parse::<f64>().ok()).is_some_and(|n| n.trunc() as i64 > limit);
            }
            if let Some(v) = o.get("attr-width-descriptor") {
                let name = v.as_str().unwrap_or("");
                return e.attr(name).is_some_and(|a| {
                    a.value_or_empty().split(',').any(|c| c.split_ascii_whitespace().nth(1).is_some_and(|d| d.ends_with('w')))
                });
            }
            if let Some(v) = o.get("ancestor") {
                let sels: Vec<&str> = v.as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
                return sels.iter().any(|s| dom.closest(i, s).is_some());
            }
            if let Some(v) = o.get("not") {
                return !eval(dom, i, v);
            }
            if let Some(Value::Array(a)) = o.get("any") {
                return a.iter().any(|c| eval(dom, i, c));
            }
            if let Some(Value::Array(a)) = o.get("all") {
                return a.iter().all(|c| eval(dom, i, c));
            }
            false
        }
        _ => false,
    }
}

/// A property of element `i` that may be a condition.
pub fn flag(dom: &Dom, meta: Meta<'_>, i: usize, key: &str) -> bool {
    meta.get(&dom.elems[i].tag, key).is_some_and(|v| eval(dom, i, v))
}

/// Evaluate a value-producing entry (`implicitRole`, `naming`).
pub fn value_of(dom: &Dom, i: usize, v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(o) => {
            if let Some(c) = o.get("if") {
                let branch = if eval(dom, i, c) { o.get("then") } else { o.get("else") };
                return branch.and_then(|b| value_of(dom, i, b));
            }
            if let Some(attr) = o.get("switch").and_then(|s| s.as_str()) {
                let cases = o.get("cases")?;
                return match dom.elems[i].value_lower(attr) {
                    Some(val) if cases.get(&val).is_some() => cases.get(&val).and_then(|c| value_of(dom, i, c)),
                    _ => o.get("default").and_then(|d| value_of(dom, i, d)),
                };
            }
            if let Some(kind) = o.get("table-cell-role").and_then(|k| k.as_str()) {
                let table_role = dom.closest(i, "table").map(|t| dom.elems[t].value_lower("role").unwrap_or_else(|| "table".into()));
                let table_role = table_role.unwrap_or_else(|| "table".into());
                return match (kind, table_role.as_str()) {
                    ("td", "table") => Some("cell".into()),
                    ("td", "grid" | "treegrid") => Some("gridcell".into()),
                    ("th", "table" | "grid" | "treegrid") => match dom.elems[i].value_lower("scope").as_deref() {
                        Some("col") => Some("columnheader".into()),
                        Some("row") => Some("rowheader".into()),
                        _ => Some(if table_role == "table" { "cell".into() } else { "gridcell".into() }),
                    },
                    _ => None,
                };
            }
            None
        }
        _ => None,
    }
}

/// The ARIA role an element has without a `role` attribute.
pub fn implicit_role(dom: &Dom, meta: Meta<'_>, i: usize) -> Option<String> {
    let aria = meta.get(&dom.elems[i].tag, "aria")?;
    value_of(dom, i, aria.get("implicitRole")?)
}

/// Whether an element may be given an accessible name (`aria-label`).
pub fn naming_allowed(dom: &Dom, meta: Meta<'_>, i: usize) -> bool {
    if let Some(role) = dom.elems[i].value_lower("role") {
        if dom.elems[i].attr("role").is_some_and(|a| a.is_dynamic()) {
            return true;
        }
        const PROHIBITED: &[&str] =
            &["caption", "code", "deletion", "emphasis", "generic", "insertion", "paragraph", "presentation", "strong", "subscript", "superscript"];
        return !PROHIBITED.contains(&role.as_str());
    }
    let Some(aria) = meta.get(&dom.elems[i].tag, "aria") else { return true };
    match aria.get("naming") {
        Some(v) => value_of(dom, i, v).is_none_or(|n| n != "prohibited"),
        None => true,
    }
}

/// Whether an element matches a content category or tag name from the
/// table (`@flow`, `li`, `legend?`), as html-validate's content model does.
/// `default` is the answer for an element the table doesn't know.
pub fn in_category(dom: &Dom, meta: Meta<'_>, i: usize, rule: &str, default: bool) -> bool {
    let raw = rule.trim_end_matches(['?', '*']);
    if !raw.starts_with('@') {
        return dom.matches(i, raw);
    }
    let tag = &dom.elems[i].tag;
    if meta.elem(tag).is_none() {
        return default;
    }
    let key = match raw {
        "@meta" => "metadata",
        "@flow" => "flow",
        "@sectioning" => "sectioning",
        "@heading" => "heading",
        "@phrasing" => "phrasing",
        "@embedded" => "embedded",
        "@interactive" => "interactive",
        "@script" => "scriptSupporting",
        "@form" => "form",
        _ => return false,
    };
    flag(dom, meta, i, key)
}

/// Whether element `i` passes a permitted-content rule list.
pub fn permitted(dom: &Dom, meta: Meta<'_>, i: usize, rules: &Value) -> bool {
    match rules {
        Value::Array(a) => a.iter().any(|r| permitted_rule(dom, meta, i, r, false)),
        _ => true,
    }
}

fn permitted_rule(dom: &Dom, meta: Meta<'_>, i: usize, rule: &Value, is_exclude: bool) -> bool {
    match rule {
        Value::String(s) => in_category(dom, meta, i, s, !is_exclude),
        Value::Array(a) => a.iter().all(|r| permitted_rule(dom, meta, i, r, is_exclude)),
        Value::Object(o) => match o.get("exclude") {
            Some(Value::Array(a)) => a.iter().all(|r| !permitted_rule(dom, meta, i, r, true)),
            Some(r) => !permitted_rule(dom, meta, i, r, true),
            None => true,
        },
        _ => true,
    }
}

// ---------------------------------------------------------------- visibility

fn attr_true(e: &Elem, name: &str) -> bool {
    e.attr(name).is_some_and(|a| a.value_or_empty().trim().eq_ignore_ascii_case("true"))
}

pub fn style_hidden(e: &Elem) -> bool {
    let Some(style) = e.attr("style").and_then(|a| a.value.as_deref()) else { return false };
    style.split(';').any(|d| {
        let Some((k, v)) = d.split_once(':') else { return false };
        let (k, v) = (k.trim().to_ascii_lowercase(), v.trim().to_ascii_lowercase());
        (k == "display" && v == "none") || (k == "visibility" && v == "hidden")
    })
}

/// `hidden` on the element or an ancestor.
pub fn html_hidden(dom: &Dom, i: usize) -> bool {
    dom.self_and_ancestors(i).any(|j| dom.elems[j].has("hidden"))
}

pub fn aria_hidden(dom: &Dom, i: usize) -> bool {
    dom.self_and_ancestors(i).any(|j| attr_true(&dom.elems[j], "aria-hidden"))
}

pub fn inert(dom: &Dom, i: usize) -> bool {
    dom.self_and_ancestors(i).any(|j| dom.elems[j].has("inert"))
}

fn presentation(dom: &Dom, meta: Meta<'_>, i: usize) -> bool {
    let e = &dom.elems[i];
    if flag(dom, meta, i, "interactive") || e.has("tabindex") {
        return false;
    }
    matches!(e.value_lower("role").as_deref(), Some("presentation" | "none"))
}

/// Whether assistive technology sees the element at all.
pub fn in_accessibility_tree(dom: &Dom, meta: Meta<'_>, i: usize) -> bool {
    !(aria_hidden(dom, i)
        || presentation(dom, meta, i)
        || html_hidden(dom, i)
        || inert(dom, i)
        || dom.self_and_ancestors(i).any(|j| style_hidden(&dom.elems[j])))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextClass {
    Empty,
    Dynamic,
    Static,
}

/// Whether an element has text, as html-validate's `classifyNodeText`:
/// `hidden` content doesn't count, nor `aria-hidden` content when
/// `accessible`.
pub fn classify_text(dom: &Dom, i: usize, accessible: bool, ignore_hidden_root: bool) -> TextClass {
    if !ignore_hidden_root && html_hidden(dom, i) {
        return TextClass::Empty;
    }
    if !ignore_hidden_root && accessible && aria_hidden(dom, i) {
        return TextClass::Empty;
    }
    if matches!(dom.elems[i].tag.as_str(), "select" | "textarea") {
        return TextClass::Empty;
    }
    let mut texts = Vec::new();
    find_texts(dom, i, accessible, &mut texts);
    if texts.iter().any(|t| is_dynamic(t)) {
        return TextClass::Dynamic;
    }
    if texts.iter().any(|t| !t.trim().is_empty()) {
        return TextClass::Static;
    }
    TextClass::Empty
}

fn find_texts<'d>(dom: &'d Dom, i: usize, accessible: bool, out: &mut Vec<&'d str>) {
    for c in &dom.elems[i].children {
        match c {
            Child::Text(t) => {
                let t = &dom.texts[*t];
                if t.kind != TextKind::Raw {
                    out.push(&dom.source[t.start..t.end]);
                }
            }
            Child::Elem(j) => {
                let e = &dom.elems[*j];
                if e.has("hidden") || (accessible && attr_true(e, "aria-hidden")) {
                    continue;
                }
                find_texts(dom, *j, accessible, out);
            }
        }
    }
}

/// `alt` present and non-empty (or template code).
pub fn has_alt_text(e: &Elem) -> bool {
    e.attr("alt").is_some_and(|a| a.value.as_deref().is_some_and(|v| !v.is_empty()))
}

pub fn has_aria_label(e: &Elem) -> bool {
    e.attr("aria-label").is_some_and(|a| a.value.as_deref().is_some_and(|v| !v.is_empty()))
}

/// The ids of an attribute holding a whitespace-separated list.
pub fn tokens_with_offsets(value: &str, base: usize) -> Vec<(usize, usize, &str)> {
    let mut out = Vec::new();
    let mut start = None;
    for (k, ch) in value.char_indices() {
        if ch.is_ascii_whitespace() {
            if let Some(s) = start.take() {
                out.push((base + s, base + k, &value[s..k]));
            }
        } else if start.is_none() {
            start = Some(k);
        }
    }
    if let Some(s) = start {
        out.push((base + s, base + value.len(), &value[s..]));
    }
    out
}

/// `html/*` rules run once per document, on the root node.
#[macro_export]
macro_rules! html_rule {
    ($ty:ident, $id:literal, |$dom:ident, $meta:ident, $ctx:ident, $out:ident| $body:block) => {
        pub struct $ty;
        impl $crate::NativeRule for $ty {
            fn id(&self) -> &'static str {
                $id
            }
            fn languages(&self) -> &'static [codelaw_core::Language] {
                &[codelaw_core::Language::Html]
            }
            fn kinds(&self) -> &'static [&'static str] {
                &["document"]
            }
            #[allow(unused_variables)]
            fn check(&self, node: codelaw_lang::SyntaxNode<'_>, $ctx: &$crate::NativeContext<'_>, $out: &mut Vec<$crate::NativeFinding>) {
                let root = node.get_inner_node();
                let $dom = $crate::html_dom::Dom::get(root, $ctx);
                let $dom: &$crate::html_dom::Dom = &$dom;
                let $meta = $crate::html_dom::Meta::from_ctx($ctx);
                $body
            }
        }
    };
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::BTreeMap;

    pub fn table() -> BTreeMap<String, Arc<Value>> {
        let elements: Value = serde_json::from_str(include_str!("../../../packs/html/data/html/elements.json")).unwrap();
        let entities: Value = serde_json::from_str(include_str!("../../../packs/html/data/html/entities.json")).unwrap();
        let mut m = BTreeMap::new();
        m.insert("html/elements".to_owned(), Arc::new(elements));
        m.insert("html/entities".to_owned(), Arc::new(entities));
        m
    }

    pub fn run(rule: &dyn crate::NativeRule, src: &str) -> Vec<NativeFinding> {
        let root = codelaw_lang::parse(codelaw_core::Language::Html, src);
        let options = BTreeMap::new();
        let data = table();
        let ctx = NativeContext { source: src, language: codelaw_core::Language::Html, semantic: None, options: &options, data: &data };
        let mut out = Vec::new();
        for node in root.root().dfs() {
            if rule.kinds().contains(&node.kind().as_ref()) {
                rule.check(node, &ctx, &mut out);
            }
        }
        out
    }

    fn build(src: &str) -> Dom {
        let root = codelaw_lang::parse(codelaw_core::Language::Html, src);
        let data = table();
        let meta = Meta { table: data.get("html/elements").map(|v| v.as_ref()) };
        let r = root.root();
        Dom::build(r.get_inner_node(), src, meta)
    }

    #[test]
    fn options_close_each_other() {
        let d = build("<select><option>a<option>b</select>");
        let sel = d.by_tag("select").next().unwrap();
        assert_eq!(d.child_elems(sel).count(), 2);
        assert_eq!(d.elems[d.child_elems(sel).next().unwrap()].closed, Closed::Implicit(ClosedBy::Sibling));
    }

    #[test]
    fn head_is_closed_by_body() {
        let d = build("<html><head><title>t</title><body><p>a</p></body></html>");
        let html = d.by_tag("html").next().unwrap();
        let kids: Vec<&str> = d.child_elems(html).map(|i| d.elems[i].tag.as_str()).collect();
        assert_eq!(kids, ["head", "body"]);
    }

    #[test]
    fn implied_head() {
        let d = build("<html><title>t</title></html>");
        let html = d.by_tag("html").next().unwrap();
        let head = d.child_elems(html).next().unwrap();
        assert!(d.elems[head].implied && d.elems[head].tag == "head");
    }

    #[test]
    fn unclosed_and_stray() {
        let d = build("<div><span>a</div></b>");
        let span = d.by_tag("span").next().unwrap();
        assert_eq!(d.elems[span].closed, Closed::Unclosed);
        assert_eq!(d.stray_end_tags.len(), 1);
    }
}
