//! Native rules for TOML in `@codelaw/rules-data`: keys and tables defined
//! twice, and the `pyproject.toml` starter rules.

use crate::util::{line_of, named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use std::collections::BTreeMap;
use tree_sitter::Node;

const TOML: &[Language] = &[Language::Toml];

/// One key part's value: a bare key as written, a quoted key decoded.
fn key_part(n: Node<'_>, src: &str) -> String {
    let t = text(n, src);
    if n.kind() != "quoted_key" || t.len() < 2 {
        return t.to_owned();
    }
    let inner = crate::util::unquote(t);
    if t.starts_with('\'') {
        return inner.to_owned();
    }
    let mut out = String::new();
    let mut it = inner.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('u') => {
                let h: String = it.by_ref().take(4).collect();
                out.extend(u32::from_str_radix(&h, 16).ok().and_then(char::from_u32));
            }
            Some('U') => {
                let h: String = it.by_ref().take(8).collect();
                out.extend(u32::from_str_radix(&h, 16).ok().and_then(char::from_u32));
            }
            Some(o) => out.push(o),
            None => {}
        }
    }
    out
}

/// A key (bare, quoted or dotted) as its parts.
pub(crate) fn key_path(n: Node<'_>, src: &str) -> Vec<String> {
    match n.kind() {
        "dotted_key" => named_children(n).into_iter().flat_map(|c| key_path(c, src)).collect(),
        "bare_key" | "quoted_key" => vec![key_part(n, src)],
        _ => Vec::new(),
    }
}

/// The key node of a table header or a pair: its first named child.
pub(crate) fn key_node(n: Node<'_>) -> Option<Node<'_>> {
    named_children(n).into_iter().find(|c| matches!(c.kind(), "bare_key" | "quoted_key" | "dotted_key"))
}

/// The value node of a pair: the named child after its key.
pub(crate) fn pair_value(pair: Node<'_>) -> Option<Node<'_>> {
    named_children(pair).into_iter().filter(|c| c.kind() != "comment").nth(1)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Def {
    /// A value: `a = 1`, inline tables and arrays included.
    Value,
    /// A table its own `[header]` defines.
    Header,
    /// A table that exists only because a longer header names it.
    ImplicitByHeader,
    /// A table a dotted key (`a.b = 1`) creates.
    Dotted,
    /// An array of tables, `[[a]]`.
    ArrayOfTables,
}

struct Defs<'a> {
    src: &'a str,
    /// Full path (with array-of-tables elements numbered) to its definition and line.
    defs: BTreeMap<Vec<String>, (Def, usize)>,
    /// Array-of-tables path to the number of elements so far.
    elements: BTreeMap<Vec<String>, usize>,
    out: &'a mut Vec<NativeFinding>,
}

impl Defs<'_> {
    /// Map a written path to the real one: a prefix that is an array of
    /// tables means its last element.
    fn resolve(&self, written: &[String]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for part in written {
            out.push(part.clone());
            if let Some(n) = self.elements.get(&out) {
                out.push(format!("[{}]", n.saturating_sub(1)));
            }
        }
        out
    }

    fn report(&mut self, at: Node<'_>, shown: &[String], line: usize, what: &str) {
        self.out.push(NativeFinding {
            start: at.start_byte(),
            end: at.end_byte(),
            message: format!("{what} `{}` is already defined on line {line}; TOML forbids defining it twice", shown.join(".")),
            edits: Vec::new(),
        });
    }

    /// `a.b.c = v` inside the table at `base`.
    fn pair(&mut self, base: &[String], pair: Node<'_>) {
        let Some(key) = key_node(pair) else { return };
        let parts = key_path(key, self.src);
        if parts.is_empty() {
            return;
        }
        let line = line_of(self.src, key.start_byte());
        let mut path = base.to_vec();
        for (i, part) in parts.iter().enumerate() {
            path.push(part.clone());
            let last = i + 1 == parts.len();
            match (self.defs.get(&path).copied(), last) {
                (None, false) => {
                    self.defs.insert(path.clone(), (Def::Dotted, line));
                }
                (Some((Def::Dotted, _)), false) => {}
                (None, true) => {
                    self.defs.insert(path.clone(), (Def::Value, line));
                }
                (Some((_, prev)), _) => {
                    self.report(key, &parts, prev, "Key");
                    return;
                }
            }
        }
        if let Some(value) = pair_value(pair)
            && value.kind() == "inline_table"
        {
            self.inline_table(value);
        }
    }

    /// Keys inside `{ … }` are their own scope.
    fn inline_table(&mut self, table: Node<'_>) {
        let mut inner = Defs { src: self.src, defs: BTreeMap::new(), elements: BTreeMap::new(), out: &mut *self.out };
        for p in named_children(table).into_iter().filter(|c| c.kind() == "pair") {
            inner.pair(&[], p);
        }
    }

    /// `[a.b]` or `[[a.b]]`; returns the path its pairs go under.
    fn header(&mut self, node: Node<'_>) -> Option<Vec<String>> {
        let key = key_node(node)?;
        let written = key_path(key, self.src);
        let line = line_of(self.src, key.start_byte());
        let array = node.kind() == "table_array_element";
        let (parent, last) = written.split_at(written.len().checked_sub(1)?);
        let mut path = self.resolve(parent);
        // Every prefix becomes a table, unless it already holds a value.
        let mut prefix: Vec<String> = Vec::new();
        for part in parent {
            prefix.push(part.clone());
            let real = self.resolve(&prefix);
            match self.defs.get(&real).copied() {
                None => {
                    self.defs.insert(real, (Def::ImplicitByHeader, line));
                }
                Some((Def::Value, prev)) => {
                    self.report(key, &prefix, prev, "Key");
                    return None;
                }
                Some(_) => {}
            }
        }
        path.push(last[0].clone());
        if array {
            match self.defs.get(&path).copied() {
                None => {
                    self.defs.insert(path.clone(), (Def::ArrayOfTables, line));
                }
                Some((Def::ArrayOfTables, _)) => {}
                Some((_, prev)) => {
                    self.report(key, &written, prev, "Table");
                    return Some(Self::detached(line));
                }
            }
            let n = self.elements.entry(path.clone()).or_insert(0);
            *n += 1;
            path.push(format!("[{}]", *n - 1));
            return Some(path);
        }
        match self.defs.get(&path).copied() {
            None | Some((Def::ImplicitByHeader, _)) => {
                self.defs.insert(path.clone(), (Def::Header, line));
                Some(path)
            }
            Some((_, prev)) => {
                self.report(key, &written, prev, "Table");
                Some(Self::detached(line))
            }
        }
    }

    /// A path no written key can reach, for the pairs of a table defined
    /// twice: they are checked among themselves, not against the first one.
    fn detached(line: usize) -> Vec<String> {
        vec![format!("\0duplicate table on line {line}")]
    }
}

pub struct NoDuplicateKeys;

impl NativeRule for NoDuplicateKeys {
    fn id(&self) -> &'static str {
        "toml/no-duplicate-keys"
    }
    fn languages(&self) -> &'static [Language] {
        TOML
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let mut defs = Defs { src: ctx.source, defs: BTreeMap::new(), elements: BTreeMap::new(), out };
        for item in named_children(node.get_inner_node()) {
            match item.kind() {
                "pair" => defs.pair(&[], item),
                "table" | "table_array_element" => {
                    let Some(base) = defs.header(item) else { continue };
                    for p in named_children(item).into_iter().filter(|c| c.kind() == "pair") {
                        defs.pair(&base, p);
                    }
                }
                _ => {}
            }
        }
    }
}

/// A `[table]` whose header is exactly `name`.
fn table<'t>(doc: Node<'t>, src: &str, name: &[&str]) -> Option<Node<'t>> {
    named_children(doc).into_iter().find(|t| t.kind() == "table" && key_node(*t).is_some_and(|k| key_path(k, src) == name))
}

/// The pairs of a table: (key path, key node, value node).
fn table_pairs<'t>(t: Node<'t>, src: &str) -> Vec<(Vec<String>, Node<'t>, Option<Node<'t>>)> {
    named_children(t)
        .into_iter()
        .filter(|p| p.kind() == "pair")
        .filter_map(|p| {
            let k = key_node(p)?;
            Some((key_path(k, src), k, pair_value(p)))
        })
        .collect()
}

/// A TOML basic or literal string's value, without escapes decoded.
pub(crate) fn string_text(n: Node<'_>, src: &str) -> Option<String> {
    if n.kind() != "string" {
        return None;
    }
    let t = text(n, src);
    let q = if t.starts_with("\"\"\"") || t.starts_with("'''") { 3 } else { 1 };
    (t.len() >= 2 * q).then(|| t[q..t.len() - q].to_owned())
}

/// A requirement's distribution name, normalised (PEP 503): `Foo_Bar` and
/// `foo-bar` sort together.
fn requirement_key(req: &str) -> (String, String) {
    let name: String = req.trim().chars().take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')).collect();
    let mut norm = String::new();
    for c in name.chars() {
        if matches!(c, '-' | '_' | '.') {
            if !norm.ends_with('-') {
                norm.push('-');
            }
        } else {
            norm.push(c.to_ascii_lowercase());
        }
    }
    (norm, req.trim().to_owned())
}

/// The finding for one unsorted array of requirement strings.
fn unsorted(key: Node<'_>, array: Node<'_>, src: &str, what: &str) -> Option<NativeFinding> {
    let items: Vec<Node> = named_children(array).into_iter().filter(|c| c.kind() != "comment").collect();
    let values: Vec<String> = items.iter().map(|i| string_text(*i, src)).collect::<Option<_>>()?;
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by(|a, b| requirement_key(&values[*a]).cmp(&requirement_key(&values[*b])).then(a.cmp(b)));
    if order.iter().enumerate().all(|(i, j)| i == *j) {
        return None;
    }
    // Comments stay on their lines while the items move, so an array with
    // comments gets no edit.
    let has_comments = named_children(array).iter().any(|c| c.kind() == "comment");
    let edits = if has_comments {
        Vec::new()
    } else {
        items.iter().zip(&order).filter(|(i, j)| i.id() != items[**j].id()).map(|(i, j)| TextEdit::new(i.start_byte(), i.end_byte(), text(items[*j], src))).collect()
    };
    Some(NativeFinding { start: key.start_byte(), end: key.end_byte(), message: format!("Sort {what} by package name"), edits })
}

pub struct DepsSorted;

impl NativeRule for DepsSorted {
    fn id(&self) -> &'static str {
        "toml/deps-sorted"
    }
    fn languages(&self) -> &'static [Language] {
        TOML
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let src = ctx.source;
        let doc = node.get_inner_node();
        if let Some(project) = table(doc, src, &["project"]) {
            for (path, key, value) in table_pairs(project, src) {
                if path == ["dependencies"]
                    && let Some(v) = value.filter(|v| v.kind() == "array")
                {
                    out.extend(unsorted(key, v, src, "`[project].dependencies`"));
                }
            }
        }
        if let Some(optional) = table(doc, src, &["project", "optional-dependencies"]) {
            for (path, key, value) in table_pairs(optional, src) {
                if let (Some(v), [extra]) = (value.filter(|v| v.kind() == "array"), path.as_slice()) {
                    out.extend(unsorted(key, v, src, &format!("the `{extra}` extra")));
                }
            }
        }
    }
}

pub struct PyprojectRequiresPython;

impl NativeRule for PyprojectRequiresPython {
    fn id(&self) -> &'static str {
        "toml/pyproject-requires-python"
    }
    fn languages(&self) -> &'static [Language] {
        TOML
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let src = ctx.source;
        let Some(project) = table(node.get_inner_node(), src, &["project"]) else { return };
        let pairs = table_pairs(project, src);
        let declared = pairs.iter().any(|(p, _, _)| p == &["requires-python"]);
        let dynamic = pairs.iter().any(|(p, _, v)| {
            p == &["dynamic"] && v.is_some_and(|v| named_children(v).iter().any(|i| string_text(*i, src).as_deref() == Some("requires-python")))
        });
        if declared || dynamic {
            return;
        }
        let range = ctx.options.get("python").and_then(|v| v.as_str()).unwrap_or(">=3.11");
        let line = format!("requires-python = {}", serde_json::to_string(range).unwrap_or_else(|_| "\">=3.11\"".into()));
        let header = key_node(project).expect("a table has a key");
        // After the table's last pair, or on the line after its header.
        let edit = match pairs.iter().rev().find_map(|(_, k, _)| k.parent()) {
            // On the line after it, so a trailing comment stays where it is.
            Some(last) => match src[last.end_byte()..].find('\n') {
                Some(i) => TextEdit::insert(last.end_byte() + i + 1, format!("{line}\n")),
                None => TextEdit::insert(src.len(), format!("\n{line}")),
            },
            None => {
                let after = src[header.end_byte()..].find('\n').map(|i| header.end_byte() + i + 1).unwrap_or(src.len());
                TextEdit::insert(after, format!("{line}\n"))
            }
        };
        out.push(NativeFinding {
            start: header.start_byte(),
            end: header.end_byte(),
            message: "`[project]` has no `requires-python`: say which Python versions the package supports".into(),
            edits: vec![edit],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(rule: &dyn NativeRule, src: &str) -> Vec<String> {
        crate::run(rule, Language::Toml, src).into_iter().map(|f| src[f.start..f.end].to_owned()).collect()
    }

    fn fixed(rule: &dyn NativeRule, src: &str) -> String {
        let mut edits: Vec<TextEdit> = crate::run(rule, Language::Toml, src).into_iter().flat_map(|f| f.edits).collect();
        edits.sort_by_key(|e| std::cmp::Reverse(e.start));
        let mut out = src.to_owned();
        for e in edits {
            out.replace_range(e.start..e.end, &e.text);
        }
        out
    }

    #[test]
    fn duplicates() {
        let src = "a = 1\na = 2\n[t]\nx.y = 1\nx.z = 2\nx = 3\n[t]\n[u.v]\n[u]\n[[w]]\nk = 1\n[[w]]\nk = 2\n[w.sub]\nq = 1\n[w.sub]\ni = { a = 1, a = 2 }\n\"i\" = 3\n";
        assert_eq!(found(&NoDuplicateKeys, src), ["a", "x", "t", "w.sub", "a", "\"i\""]);
    }

    #[test]
    fn deps_sorted() {
        let src = "[project]\ndependencies = [\n  \"requests>=2\",\n  \"Click\",\n  \"attrs\",\n]\n[project.optional-dependencies]\ndev = [\"pytest\", \"black\"]\n";
        assert_eq!(
            fixed(&DepsSorted, src),
            "[project]\ndependencies = [\n  \"attrs\",\n  \"Click\",\n  \"requests>=2\",\n]\n[project.optional-dependencies]\ndev = [\"black\", \"pytest\"]\n"
        );
    }

    #[test]
    fn requires_python() {
        assert_eq!(fixed(&PyprojectRequiresPython, "[project]\nname = \"x\"\n\n[tool.x]\n"), "[project]\nname = \"x\"\nrequires-python = \">=3.11\"\n\n[tool.x]\n");
        assert!(found(&PyprojectRequiresPython, "[project]\ndynamic = [\"requires-python\"]\n").is_empty());
    }
}
