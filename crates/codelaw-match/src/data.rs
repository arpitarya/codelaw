//! The data tree of a JSON, YAML or TOML file: the values the file holds,
//! each with the byte span it was written at, so a `target:` rule can name a
//! path and a fix can edit exactly the text behind it (SR-RULE-FORMAT
//! decision 3e).
//!
//! The tree is built from the tree-sitter syntax tree the engine already
//! parsed; nothing is re-read. YAML is read as YAML 1.2 (`on` and `yes` are
//! strings); an alias reads as its anchor's value and a `<<` merge key
//! merges its mapping. TOML dates and times read as strings, as they do in
//! JSON Schema tools.

use codelaw_core::Language;
use std::collections::HashMap;
use tree_sitter::Node;

pub type NodeId = usize;

/// A number as written: its value, and whether it is an integer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Number {
    pub value: f64,
    pub integer: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<NodeId>),
    Object(Vec<Member>),
}

/// One key of an object and where it was written.
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub key: String,
    /// The key as written; `None` for a key no text spells on its own (a
    /// TOML table a dotted header implies).
    pub key_span: Option<(usize, usize)>,
    pub value: NodeId,
    /// The whole member: key, separator and value (a YAML or TOML pair, a
    /// JSON pair). `None` when the member is not one contiguous piece of
    /// text: a TOML table, a key written by several dotted keys, a merged
    /// YAML key.
    pub span: Option<(usize, usize)>,
}

/// How a container is written, which is what a fix needs to edit it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Scalar,
    JsonObject,
    JsonArray,
    YamlBlockMapping,
    YamlFlowMapping,
    YamlBlockSequence,
    YamlFlowSequence,
    /// The TOML document's top level, or a `[table]` section: `end` is where
    /// its last pair (or its header) ends.
    TomlSection {
        header: Option<(usize, usize)>,
        end: usize,
    },
    TomlInlineTable,
    TomlArray,
    /// A TOML table no text defines on its own (`[a.b]` implies `a`), or an
    /// array of tables.
    TomlImplicit,
    /// A YAML value that is an alias or holds merged keys: its text is
    /// elsewhere.
    Borrowed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataNode {
    pub value: Value,
    /// The value's text.
    pub span: (usize, usize),
    pub style: Style,
}

/// The values of one file. A YAML stream may hold several documents.
#[derive(Clone, Debug)]
pub struct DataTree {
    pub language: Language,
    pub nodes: Vec<DataNode>,
    pub documents: Vec<NodeId>,
}

impl DataTree {
    pub fn node(&self, id: NodeId) -> &DataNode {
        &self.nodes[id]
    }

    /// The member of an object named `key` (the last one, if written twice).
    pub fn member(&self, id: NodeId, key: &str) -> Option<&Member> {
        match &self.nodes[id].value {
            Value::Object(ms) => ms.iter().rev().find(|m| m.key == key),
            _ => None,
        }
    }

    /// The value as JSON, for comparing with a rule's `equals:`, an `enum`,
    /// or the document before and after a fix.
    pub fn to_json(&self, id: NodeId) -> serde_json::Value {
        self.to_json_depth(id, 0)
    }

    fn to_json_depth(&self, id: NodeId, depth: usize) -> serde_json::Value {
        use serde_json::Value as J;
        if depth > 256 {
            return J::Null;
        }
        match &self.nodes[id].value {
            Value::Null => J::Null,
            Value::Bool(b) => J::Bool(*b),
            Value::Number(n) => number_json(*n),
            Value::String(s) => J::String(s.clone()),
            Value::Array(items) => J::Array(items.iter().map(|i| self.to_json_depth(*i, depth + 1)).collect()),
            Value::Object(ms) => {
                let mut map = serde_json::Map::new();
                for m in ms {
                    map.insert(m.key.clone(), self.to_json_depth(m.value, depth + 1));
                }
                J::Object(map)
            }
        }
    }
}

pub fn number_json(n: Number) -> serde_json::Value {
    if n.integer && n.value.abs() < 9.0e15 {
        serde_json::Value::from(n.value as i64)
    } else {
        serde_json::Number::from_f64(n.value).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null)
    }
}

/// Build the data tree of a parsed JSON, YAML or TOML file; `None` for any
/// other language.
pub fn build(language: Language, root: tree_sitter::Node<'_>, src: &str) -> Option<DataTree> {
    let mut b = Builder { src, nodes: Vec::new(), anchors: HashMap::new() };
    let documents = match language {
        Language::Json => named(root).into_iter().filter(|n| n.kind() != "comment").take(1).map(|n| b.json(n)).collect(),
        Language::Yaml => {
            let docs: Vec<Node> = if root.kind() == "stream" { named(root).into_iter().filter(|n| n.kind() == "document").collect() } else { vec![root] };
            docs.into_iter().filter_map(|d| named(d).into_iter().find(|n| matches!(n.kind(), "block_node" | "flow_node")).map(|n| b.yaml(n))).collect()
        }
        Language::Toml => vec![b.toml(root)],
        _ => return None,
    };
    Some(DataTree { language, nodes: b.nodes, documents })
}

pub(crate) fn named(n: Node<'_>) -> Vec<Node<'_>> {
    let mut c = n.walk();
    n.named_children(&mut c).collect()
}

fn span(n: Node<'_>) -> (usize, usize) {
    (n.start_byte(), n.end_byte())
}

struct Builder<'s> {
    src: &'s str,
    nodes: Vec<DataNode>,
    anchors: HashMap<String, NodeId>,
}

impl Builder<'_> {
    fn push(&mut self, value: Value, span: (usize, usize), style: Style) -> NodeId {
        self.nodes.push(DataNode { value, span, style });
        self.nodes.len() - 1
    }

    fn text(&self, n: Node<'_>) -> &str {
        &self.src[n.byte_range()]
    }

    // ---- JSON ----

    fn json(&mut self, n: Node<'_>) -> NodeId {
        let sp = span(n);
        match n.kind() {
            "object" => {
                let mut members = Vec::new();
                for p in named(n).into_iter().filter(|p| p.kind() == "pair") {
                    let (Some(k), Some(v)) = (p.child_by_field_name("key"), p.child_by_field_name("value")) else { continue };
                    let key = json_string(self.text(k));
                    let value = self.json(v);
                    members.push(Member { key, key_span: Some(span(k)), value, span: Some(span(p)) });
                }
                self.push(Value::Object(members), sp, Style::JsonObject)
            }
            "array" => {
                let items = named(n).into_iter().filter(|c| c.kind() != "comment").map(|c| self.json(c)).collect();
                self.push(Value::Array(items), sp, Style::JsonArray)
            }
            "string" => {
                let s = json_string(self.text(n));
                self.push(Value::String(s), sp, Style::Scalar)
            }
            "number" => {
                let t = self.text(n);
                let integer = !t.contains(['.', 'e', 'E']);
                self.push(Value::Number(Number { value: t.parse().unwrap_or(0.0), integer }), sp, Style::Scalar)
            }
            "true" => self.push(Value::Bool(true), sp, Style::Scalar),
            "false" => self.push(Value::Bool(false), sp, Style::Scalar),
            _ => self.push(Value::Null, sp, Style::Scalar),
        }
    }

    // ---- YAML ----

    fn yaml(&mut self, n: Node<'_>) -> NodeId {
        let sp = span(n);
        match n.kind() {
            "block_node" | "flow_node" => {
                let kids = named(n);
                let mut anchor = None;
                let mut tag = None;
                let mut content = None;
                for k in &kids {
                    match k.kind() {
                        "anchor" => anchor = named(*k).into_iter().find(|c| c.kind() == "anchor_name").map(|a| self.text(a).to_owned()),
                        "tag" => tag = Some(self.text(*k).to_owned()),
                        "comment" => {}
                        _ => content = Some(*k),
                    }
                }
                let id = match content {
                    Some(c) if tag.as_deref() == Some("!!str") && matches!(c.kind(), "plain_scalar") => {
                        let t = self.text(c).trim().to_owned();
                        self.push(Value::String(t), span(c), Style::Scalar)
                    }
                    Some(c) => self.yaml(c),
                    None => self.push(Value::Null, sp, Style::Scalar),
                };
                if let Some(a) = anchor {
                    self.anchors.insert(a, id);
                }
                id
            }
            "block_mapping" | "flow_mapping" => {
                let style = if n.kind() == "block_mapping" { Style::YamlBlockMapping } else { Style::YamlFlowMapping };
                let mut members = Vec::new();
                let mut merged = false;
                for p in named(n).into_iter().filter(|p| matches!(p.kind(), "block_mapping_pair" | "flow_pair" | "flow_node")) {
                    if p.kind() == "flow_node" {
                        // `{a}`: a key with no value.
                        let key = self.yaml_key(p);
                        let value = self.push(Value::Null, (p.end_byte(), p.end_byte()), Style::Scalar);
                        members.push(Member { key, key_span: Some(span(p)), value, span: Some(span(p)) });
                        continue;
                    }
                    let k = p.child_by_field_name("key");
                    let v = p.child_by_field_name("value");
                    let key = k.map(|k| self.yaml_key(k)).unwrap_or_default();
                    let value = match v {
                        Some(v) => self.yaml(v),
                        None => {
                            let at = k.map(|k| k.end_byte()).unwrap_or(p.end_byte());
                            self.push(Value::Null, (at, at), Style::Scalar)
                        }
                    };
                    if key == "<<" && k.is_some_and(|k| self.is_plain(k)) {
                        merged = true;
                        self.merge(value, &mut members);
                        continue;
                    }
                    members.push(Member { key, key_span: k.map(span), value, span: Some(trimmed(self.src, span(p))) });
                }
                self.push(Value::Object(members), sp, if merged { Style::Borrowed } else { style })
            }
            "block_sequence" => {
                let items = named(n)
                    .into_iter()
                    .filter(|c| c.kind() == "block_sequence_item")
                    .map(|item| match named(item).into_iter().find(|c| c.kind() != "comment") {
                        Some(v) => self.yaml(v),
                        None => self.push(Value::Null, (item.end_byte(), item.end_byte()), Style::Scalar),
                    })
                    .collect();
                self.push(Value::Array(items), sp, Style::YamlBlockSequence)
            }
            "flow_sequence" => {
                let mut items = Vec::new();
                for c in named(n).into_iter().filter(|c| c.kind() != "comment") {
                    if c.kind() == "flow_pair" {
                        // `[a: 1]` is a one-pair mapping.
                        let k = c.child_by_field_name("key");
                        let key = k.map(|k| self.yaml_key(k)).unwrap_or_default();
                        let value = match c.child_by_field_name("value") {
                            Some(v) => self.yaml(v),
                            None => self.push(Value::Null, (c.end_byte(), c.end_byte()), Style::Scalar),
                        };
                        let m = Member { key, key_span: k.map(span), value, span: Some(span(c)) };
                        items.push(self.push(Value::Object(vec![m]), span(c), Style::Borrowed));
                    } else {
                        items.push(self.yaml(c));
                    }
                }
                self.push(Value::Array(items), sp, Style::YamlFlowSequence)
            }
            "alias" => {
                let name = named(n).into_iter().find(|c| c.kind() == "alias_name").map(|a| self.text(a).to_owned()).unwrap_or_default();
                match self.anchors.get(&name) {
                    Some(&target) => {
                        let value = self.nodes[target].value.clone();
                        self.push(value, sp, Style::Borrowed)
                    }
                    None => self.push(Value::Null, sp, Style::Borrowed),
                }
            }
            "plain_scalar" => {
                let inner = named(n).into_iter().next();
                let t = self.text(n).trim().to_owned();
                let value = match inner.map(|i| i.kind()) {
                    Some("null_scalar") => Value::Null,
                    Some("boolean_scalar") => Value::Bool(matches!(t.as_str(), "true" | "True" | "TRUE")),
                    Some("integer_scalar") => match yaml_int(&t) {
                        Some(v) => Value::Number(Number { value: v, integer: true }),
                        None => Value::String(t),
                    },
                    Some("float_scalar") => match yaml_float(&t) {
                        Some(v) => Value::Number(Number { value: v, integer: false }),
                        None => Value::String(t),
                    },
                    _ => Value::String(fold_plain(&t)),
                };
                self.push(value, sp, Style::Scalar)
            }
            "double_quote_scalar" => {
                let t = self.text(n);
                let s = yaml_double(&t[1..t.len().saturating_sub(1).max(1)]);
                self.push(Value::String(s), sp, Style::Scalar)
            }
            "single_quote_scalar" => {
                let t = self.text(n);
                let s = fold_plain(&t[1..t.len().saturating_sub(1).max(1)].replace("''", "'"));
                self.push(Value::String(s), sp, Style::Scalar)
            }
            "block_scalar" => {
                let s = block_scalar(self.text(n));
                self.push(Value::String(s), sp, Style::Scalar)
            }
            _ => {
                let t = self.text(n).to_owned();
                self.push(Value::String(t), sp, Style::Scalar)
            }
        }
    }

    fn is_plain(&self, key: Node<'_>) -> bool {
        named(key).first().is_some_and(|c| c.kind() == "plain_scalar")
    }

    /// A YAML key's text: a scalar's value; any other key as written.
    fn yaml_key(&mut self, key: Node<'_>) -> String {
        let save = self.nodes.len();
        let id = self.yaml(key);
        let out = match &self.nodes[id].value {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            Value::Bool(b) => b.to_string(),
            Value::Number(_) => self.text(key).trim().to_owned(),
            _ => self.text(key).trim().to_owned(),
        };
        // A key is not a value of the tree; drop what reading it added,
        // unless it set an anchor.
        if !self.anchors.values().any(|&a| a >= save) {
            self.nodes.truncate(save);
        }
        out
    }

    /// `<<: *base` or `<<: [*a, *b]`: the mapping's keys, unless the map
    /// sets them itself.
    fn merge(&mut self, value: NodeId, members: &mut Vec<Member>) {
        let sources: Vec<NodeId> = match &self.nodes[value].value {
            Value::Object(_) => vec![value],
            Value::Array(items) => items.clone(),
            _ => Vec::new(),
        };
        for s in sources {
            if let Value::Object(ms) = self.nodes[s].value.clone() {
                for m in ms {
                    if !members.iter().any(|x| x.key == m.key) {
                        members.push(Member { span: None, ..m });
                    }
                }
            }
        }
    }

    // ---- TOML ----

    fn toml(&mut self, doc: Node<'_>) -> NodeId {
        let root_pairs: Vec<Node> = named(doc).into_iter().filter(|n| n.kind() == "pair").collect();
        let root_end = root_pairs.last().map(|p| p.end_byte()).unwrap_or(0);
        let root = self.push(Value::Object(Vec::new()), span(doc), Style::TomlSection { header: None, end: root_end });
        for p in root_pairs {
            self.toml_pair(root, p);
        }
        for t in named(doc).into_iter().filter(|n| matches!(n.kind(), "table" | "table_array_element")) {
            let Some(key) = toml_key_node(t) else { continue };
            let path = toml_key_path(key, self.src);
            if path.is_empty() {
                continue;
            }
            let pairs: Vec<Node> = named(t).into_iter().filter(|n| n.kind() == "pair").collect();
            let header = (t.start_byte(), self.src[t.start_byte()..].find('\n').map(|i| t.start_byte() + i).unwrap_or(t.end_byte()));
            let end = pairs.last().map(|p| p.end_byte()).unwrap_or(header.1);
            let parent = self.toml_walk(root, &path[..path.len() - 1]);
            let last = path.last().expect("not empty").clone();
            let section = if t.kind() == "table" {
                match self.find_member(parent, &last) {
                    // `[a]` after `[a.b]` defines `a` itself now.
                    Some(existing) if matches!(self.nodes[existing].style, Style::TomlImplicit) => {
                        self.nodes[existing].style = Style::TomlSection { header: Some(header), end };
                        self.nodes[existing].span = span(t);
                        self.set_member_key_span(parent, &last, Some(span(key)));
                        existing
                    }
                    Some(existing) => existing,
                    None => {
                        let id = self.push(Value::Object(Vec::new()), span(t), Style::TomlSection { header: Some(header), end });
                        self.add_member(parent, Member { key: last, key_span: Some(span(key)), value: id, span: None });
                        id
                    }
                }
            } else {
                let array = match self.find_member(parent, &last) {
                    Some(a) => a,
                    None => {
                        let a = self.push(Value::Array(Vec::new()), span(t), Style::TomlImplicit);
                        self.add_member(parent, Member { key: last, key_span: Some(span(key)), value: a, span: None });
                        a
                    }
                };
                let id = self.push(Value::Object(Vec::new()), span(t), Style::TomlSection { header: Some(header), end });
                if let Value::Array(items) = &mut self.nodes[array].value {
                    items.push(id);
                }
                id
            };
            for p in pairs {
                self.toml_pair(section, p);
            }
        }
        root
    }

    fn find_member(&self, obj: NodeId, key: &str) -> Option<NodeId> {
        match &self.nodes[obj].value {
            Value::Object(ms) => ms.iter().rev().find(|m| m.key == key).map(|m| m.value),
            _ => None,
        }
    }

    fn add_member(&mut self, obj: NodeId, m: Member) {
        if let Value::Object(ms) = &mut self.nodes[obj].value {
            ms.push(m);
        }
    }

    fn set_member_key_span(&mut self, obj: NodeId, key: &str, ks: Option<(usize, usize)>) {
        if let Value::Object(ms) = &mut self.nodes[obj].value
            && let Some(m) = ms.iter_mut().rev().find(|m| m.key == key)
        {
            m.key_span = ks;
        }
    }

    /// The table at `path` under `from`, made implicitly where missing. The
    /// last table of an array of tables is the one a path walks into.
    fn toml_walk(&mut self, from: NodeId, path: &[String]) -> NodeId {
        let mut cur = from;
        for k in path {
            let next = match self.find_member(cur, k) {
                Some(n) => match &self.nodes[n].value {
                    Value::Array(items) if matches!(self.nodes[n].style, Style::TomlImplicit) => *items.last().unwrap_or(&n),
                    _ => n,
                },
                None => {
                    let sp = self.nodes[cur].span;
                    let id = self.push(Value::Object(Vec::new()), sp, Style::TomlImplicit);
                    self.add_member(cur, Member { key: k.clone(), key_span: None, value: id, span: None });
                    id
                }
            };
            cur = next;
        }
        cur
    }

    fn toml_pair(&mut self, table: NodeId, p: Node<'_>) {
        let kids: Vec<Node> = named(p).into_iter().filter(|c| c.kind() != "comment").collect();
        let (Some(key), Some(v)) = (kids.first().copied(), kids.get(1).copied()) else { return };
        let path = toml_key_path(key, self.src);
        if path.is_empty() {
            return;
        }
        let value = self.toml_value(v);
        if path.len() == 1 {
            let sp = trimmed(self.src, (p.start_byte(), v.end_byte()));
            self.add_member(table, Member { key: path[0].clone(), key_span: Some(span(key)), value, span: Some(sp) });
        } else {
            // `a.b = 1` defines `a` implicitly; its key is not one piece.
            let parent = self.toml_walk(table, &path[..path.len() - 1]);
            let last_key = named(key).last().copied().unwrap_or(key);
            self.add_member(parent, Member { key: path.last().expect("not empty").clone(), key_span: Some(span(last_key)), value, span: None });
        }
    }

    fn toml_value(&mut self, v: Node<'_>) -> NodeId {
        let sp = span(v);
        let t = self.text(v).to_owned();
        match v.kind() {
            "string" => self.push(Value::String(toml_string(&t)), sp, Style::Scalar),
            "integer" => self.push(Value::Number(Number { value: toml_int(&t).unwrap_or(0.0), integer: true }), sp, Style::Scalar),
            "float" => {
                let c = t.replace('_', "");
                let value = match c.trim_start_matches(['+', '-']) {
                    "inf" => f64::INFINITY * if c.starts_with('-') { -1.0 } else { 1.0 },
                    "nan" => f64::NAN,
                    _ => c.parse().unwrap_or(0.0),
                };
                self.push(Value::Number(Number { value, integer: false }), sp, Style::Scalar)
            }
            "boolean" => self.push(Value::Bool(t == "true"), sp, Style::Scalar),
            "array" => {
                let items = named(v).into_iter().filter(|c| c.kind() != "comment").map(|c| self.toml_value(c)).collect();
                self.push(Value::Array(items), sp, Style::TomlArray)
            }
            "inline_table" => {
                let id = self.push(Value::Object(Vec::new()), sp, Style::TomlInlineTable);
                for p in named(v).into_iter().filter(|c| c.kind() == "pair") {
                    self.toml_pair(id, p);
                }
                id
            }
            _ => self.push(Value::String(t), sp, Style::Scalar),
        }
    }
}

/// A span without the whitespace it ends with.
fn trimmed(src: &str, (s, e): (usize, usize)) -> (usize, usize) {
    let t = src[s..e].trim_end();
    (s, s + t.len())
}

/// A JSON string literal's value, escapes decoded.
pub fn json_string(t: &str) -> String {
    let inner = if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') { &t[1..t.len() - 1] } else { t };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    let mut pending_high: Option<u32> = None;
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some('u') => {
                let hex: String = chars.by_ref().take(4).collect();
                let Ok(v) = u32::from_str_radix(&hex, 16) else { continue };
                if (0xD800..0xDC00).contains(&v) {
                    pending_high = Some(v);
                    continue;
                }
                let v = match pending_high.take() {
                    Some(h) if (0xDC00..0xE000).contains(&v) => 0x10000 + ((h - 0xD800) << 10) + (v - 0xDC00),
                    _ => v,
                };
                out.extend(char::from_u32(v));
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

fn yaml_double(inner: &str) -> String {
    let mut out = String::new();
    let mut it = inner.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let hex = |it: &mut std::iter::Peekable<std::str::Chars<'_>>, n: usize| {
            let h: String = it.by_ref().take(n).collect();
            u32::from_str_radix(&h, 16).ok().and_then(char::from_u32)
        };
        match it.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('0') => out.push('\0'),
            Some('x') => out.extend(hex(&mut it, 2)),
            Some('u') => out.extend(hex(&mut it, 4)),
            Some('U') => out.extend(hex(&mut it, 8)),
            Some('\n') => {
                while it.peek().is_some_and(|c| *c == ' ' || *c == '\t') {
                    it.next();
                }
            }
            Some(o) => out.push(o),
            None => {}
        }
    }
    fold_plain(&out)
}

/// A flow scalar over several lines folds each line break into a space.
fn fold_plain(t: &str) -> String {
    if !t.contains('\n') {
        return t.to_owned();
    }
    let lines: Vec<&str> = t.split('\n').map(str::trim).collect();
    let mut out = String::new();
    let mut blank = 0;
    for (i, l) in lines.iter().enumerate() {
        if l.is_empty() && i > 0 {
            blank += 1;
            continue;
        }
        if i > 0 {
            if blank > 0 {
                out.push_str(&"\n".repeat(blank));
            } else {
                out.push(' ');
            }
        }
        blank = 0;
        out.push_str(l);
    }
    out
}

/// A `|` or `>` block scalar's value.
fn block_scalar(t: &str) -> String {
    let mut lines = t.split('\n');
    let header = lines.next().unwrap_or_default();
    let folded = header.starts_with('>');
    let keep = header.contains('+');
    let strip = header.contains('-');
    let body: Vec<&str> = lines.collect();
    let indent = body.iter().filter(|l| !l.trim().is_empty()).map(|l| l.len() - l.trim_start_matches(' ').len()).min().unwrap_or(0);
    let content: Vec<&str> = body.iter().map(|l| if l.len() >= indent { &l[indent..] } else { "" }).collect();
    let mut out = String::new();
    if folded {
        let mut prev_blank = true;
        for l in &content {
            if l.is_empty() {
                out.push('\n');
                prev_blank = true;
            } else {
                if !prev_blank && !l.starts_with(' ') {
                    out.push(' ');
                } else if !prev_blank {
                    out.push('\n');
                }
                out.push_str(l);
                prev_blank = false;
            }
        }
    } else {
        out = content.join("\n");
    }
    let body = out.trim_end_matches('\n').to_owned();
    if strip {
        body
    } else if keep {
        out
    } else {
        body + "\n"
    }
}

fn yaml_int(t: &str) -> Option<f64> {
    let (neg, d) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let v = if let Some(h) = d.strip_prefix("0x") {
        i64::from_str_radix(h, 16).ok()? as f64
    } else if let Some(o) = d.strip_prefix("0o") {
        i64::from_str_radix(o, 8).ok()? as f64
    } else {
        d.parse::<f64>().ok()?
    };
    Some(if neg { -v } else { v })
}

fn yaml_float(t: &str) -> Option<f64> {
    match t.trim_start_matches(['+', '-']).to_ascii_lowercase().as_str() {
        ".inf" => Some(if t.starts_with('-') { f64::NEG_INFINITY } else { f64::INFINITY }),
        ".nan" => Some(f64::NAN),
        _ => t.parse().ok(),
    }
}

fn toml_int(t: &str) -> Option<f64> {
    let c = t.replace('_', "");
    let (neg, d) = match c.strip_prefix('-') {
        Some(r) => (true, r.to_owned()),
        None => (false, c.trim_start_matches('+').to_owned()),
    };
    let v = if let Some(h) = d.strip_prefix("0x") {
        i64::from_str_radix(h, 16).ok()? as f64
    } else if let Some(o) = d.strip_prefix("0o") {
        i64::from_str_radix(o, 8).ok()? as f64
    } else if let Some(b) = d.strip_prefix("0b") {
        i64::from_str_radix(b, 2).ok()? as f64
    } else {
        d.parse::<f64>().ok()?
    };
    Some(if neg { -v } else { v })
}

/// A TOML string's value: basic and multi-line basic strings with escapes
/// decoded, literal strings as written.
pub fn toml_string(t: &str) -> String {
    let (q, literal) = if t.starts_with("\"\"\"") {
        (3, false)
    } else if t.starts_with("'''") {
        (3, true)
    } else {
        (1, t.starts_with('\''))
    };
    if t.len() < 2 * q {
        return t.to_owned();
    }
    let mut inner = &t[q..t.len() - q];
    if q == 3 {
        inner = inner.strip_prefix("\r\n").or_else(|| inner.strip_prefix('\n')).unwrap_or(inner);
    }
    if literal {
        return inner.to_owned();
    }
    let mut out = String::new();
    let mut it = inner.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let hex = |it: &mut std::iter::Peekable<std::str::Chars<'_>>, n: usize| {
            let h: String = it.by_ref().take(n).collect();
            u32::from_str_radix(&h, 16).ok().and_then(char::from_u32)
        };
        match it.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some('e') => out.push('\u{1b}'),
            Some('u') => out.extend(hex(&mut it, 4)),
            Some('U') => out.extend(hex(&mut it, 8)),
            // A line-ending backslash trims the break and the indentation.
            Some(w) if w.is_whitespace() => {
                while it.peek().is_some_and(|c| c.is_whitespace()) {
                    it.next();
                }
            }
            Some(o) => out.push(o),
            None => {}
        }
    }
    out
}

fn toml_key_node(n: Node<'_>) -> Option<Node<'_>> {
    named(n).into_iter().find(|c| matches!(c.kind(), "bare_key" | "quoted_key" | "dotted_key"))
}

fn toml_key_path(n: Node<'_>, src: &str) -> Vec<String> {
    match n.kind() {
        "dotted_key" => named(n).into_iter().flat_map(|c| toml_key_path(c, src)).collect(),
        "bare_key" => vec![src[n.byte_range()].to_owned()],
        "quoted_key" => vec![toml_string(&src[n.byte_range()])],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(language: Language, src: &str) -> (DataTree, serde_json::Value) {
        let root = codelaw_lang::parse(language, src);
        let t = build(language, root.root().get_inner_node(), src).unwrap();
        let j = t.documents.first().map(|d| t.to_json(*d)).unwrap_or_default();
        (t, j)
    }

    #[test]
    fn json_values() {
        let (_, j) = tree(Language::Json, "{\n  // c\n  \"a\": [1, 2.5, true, null, \"s\\u0041\"],\n  \"b\": {\"c\": {}}\n}\n");
        assert_eq!(j, serde_json::json!({"a": [1, 2.5, true, null, "sA"], "b": {"c": {}}}));
    }

    #[test]
    fn yaml_values() {
        let src = "name: x\non:\n  push:\n    branches: [main, \"dev\"]\nbase: &b\n  k: 1\n  f: 1.5\n  n: null\n  t: true\n  y: yes\n  s: |\n    hi\n    there\n  e:\ncopy: *b\nmerged:\n  <<: *b\n  k: 2\n";
        let (_, j) = tree(Language::Yaml, src);
        let base = serde_json::json!({"k": 1, "f": 1.5, "n": null, "t": true, "y": "yes", "s": "hi\nthere\n", "e": null});
        let mut merged = base.clone();
        merged["k"] = serde_json::json!(2);
        assert_eq!(j["on"], serde_json::json!({"push": {"branches": ["main", "dev"]}}));
        assert_eq!(j["base"], base);
        assert_eq!(j["copy"], base);
        assert_eq!(j["merged"], merged);
    }

    #[test]
    fn toml_values() {
        let src = "# top\nname = \"x\"\n[project]\nname = 'p' # tr\ndeps = [\"b\", \"a\"]\nopt = { a = 1, b.c = 2 }\na.b = 3\n[tool.ruff]\nx = 1979-05-27\n[[arr]]\nq = 1\n[[arr]]\nq = 0x10\n";
        let (t, j) = tree(Language::Toml, src);
        assert_eq!(
            j,
            serde_json::json!({
                "name": "x",
                "project": {"name": "p", "deps": ["b", "a"], "opt": {"a": 1, "b": {"c": 2}}, "a": {"b": 3}},
                "tool": {"ruff": {"x": "1979-05-27"}},
                "arr": [{"q": 1}, {"q": 16}]
            })
        );
        let project = t.member(t.documents[0], "project").unwrap().value;
        assert!(matches!(t.node(project).style, Style::TomlSection { header: Some(_), .. }));
        let name = t.member(project, "name").unwrap();
        assert_eq!(&src[name.span.unwrap().0..name.span.unwrap().1], "name = 'p'");
    }
}
