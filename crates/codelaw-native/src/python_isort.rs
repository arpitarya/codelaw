//! `py/unsorted-imports` (Ruff's `I001`, isort's ordering) and
//! `py/all-sorted` (Ruff's `RUF022`): import blocks and `__all__` in a
//! fixed order, each with an `auto` fix.

use crate::python_pyflakes::dunder_all_strings;
use crate::python_util::{hit_fix, indent_at, kids, py_rule, statements, string_value};
use crate::util::text;
use crate::{NativeContext, NativeFinding};
use codelaw_core::TextEdit;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use tree_sitter::Node;

/// Natural, case-insensitive order (`a2` before `a10`), then case-sensitive.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    natural(&a.to_lowercase(), &b.to_lowercase()).then_with(|| natural(a, b))
}

fn natural(a: &str, b: &str) -> Ordering {
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let mut n1 = String::new();
                while let Some(c) = x.peek().copied().filter(|c| c.is_ascii_digit()) {
                    n1.push(c);
                    x.next();
                }
                let mut n2 = String::new();
                while let Some(d) = y.peek().copied().filter(|c| c.is_ascii_digit()) {
                    n2.push(d);
                    y.next();
                }
                let t1 = n1.trim_start_matches('0');
                let t2 = n2.trim_start_matches('0');
                let o = t1.len().cmp(&t2.len()).then_with(|| t1.cmp(t2));
                if o != Ordering::Equal {
                    return o;
                }
            }
            (Some(c), Some(d)) => {
                if c != d {
                    return c.cmp(&d);
                }
                x.next();
                y.next();
            }
        }
    }
}

/// isort's `order-by-type`: CONSTANTS, then Classes, then everything else.
fn member_rank(name: &str) -> u8 {
    let cased: Vec<char> = name.chars().filter(|c| c.is_alphabetic()).collect();
    if name.chars().count() > 1 && !cased.is_empty() && cased.iter().all(|c| c.is_uppercase()) {
        0
    } else if name.chars().next().is_some_and(|c| c.is_uppercase()) {
        1
    } else {
        2
    }
}

pub fn member_cmp(a: &str, b: &str) -> Ordering {
    member_rank(a).cmp(&member_rank(b)).then_with(|| natural_cmp(a, b))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    Straight,
    From,
}

#[derive(Clone, Debug)]
struct Unit {
    kind: Kind,
    /// The module: `os.path`, `.`, `..pkg`.
    module: String,
    /// `import a as b`'s alias.
    alias: Option<String>,
    /// `from m import` members: (name, alias).
    members: Vec<(String, Option<String>)>,
    star: bool,
    leading: Vec<String>,
    trailing: Option<String>,
    vertical: bool,
    /// Comments inside the statement, which a rewrite would lose.
    inner_comments: bool,
}

impl Unit {
    fn same(&self, o: &Unit) -> bool {
        self.kind == o.kind && self.module == o.module && self.alias == o.alias && self.members == o.members && self.star == o.star
    }
}

fn section(u: &Unit, stdlib: &BTreeSet<String>, first_party: &BTreeSet<String>) -> u8 {
    if u.module == "__future__" {
        return 0;
    }
    if u.module.starts_with('.') {
        return 4;
    }
    let top = u.module.split('.').next().unwrap_or("");
    if first_party.contains(top) {
        3
    } else if stdlib.contains(top) {
        1
    } else {
        2
    }
}

fn module_cmp(a: &Unit, b: &Unit) -> Ordering {
    let dots = |m: &str| m.chars().take_while(|c| *c == '.').count();
    // Relative imports: furthest first.
    dots(&b.module)
        .cmp(&dots(&a.module))
        .then_with(|| natural_cmp(a.module.trim_start_matches('.'), b.module.trim_start_matches('.')))
}

fn unit_cmp(a: &Unit, b: &Unit) -> Ordering {
    let kind = |u: &Unit| if u.kind == Kind::Straight { 0 } else { 1 };
    kind(a)
        .cmp(&kind(b))
        .then_with(|| module_cmp(a, b))
        .then_with(|| a.alias.is_some().cmp(&b.alias.is_some()))
        .then_with(|| natural_cmp(a.alias.as_deref().unwrap_or(""), b.alias.as_deref().unwrap_or("")))
        .then_with(|| a.star.cmp(&b.star).reverse())
        .then_with(|| match (a.members.first(), b.members.first()) {
            (Some(x), Some(y)) => member_cmp(&x.0, &y.0),
            _ => Ordering::Equal,
        })
        .then_with(|| {
            let aliased = |u: &Unit| u.members.iter().any(|m| m.1.is_some());
            aliased(a).cmp(&aliased(b))
        })
}

/// One import statement as units. `None` when it can't be read safely.
fn units_of(stmt: Node<'_>, src: &str, leading: Vec<String>, trailing: Option<String>) -> Option<Vec<Unit>> {
    let inner_comments = kids(stmt).iter().any(|c| c.kind() == "comment") || {
        let mut stack = vec![stmt];
        let mut found = false;
        while let Some(n) = stack.pop() {
            if n.kind() == "comment" && n.id() != stmt.id() {
                found = true;
            }
            stack.extend(crate::util::named_children(n));
        }
        found
    };
    let blank = |kind: Kind, module: String| Unit {
        kind,
        module,
        alias: None,
        members: Vec::new(),
        star: false,
        leading: Vec::new(),
        trailing: None,
        vertical: false,
        inner_comments,
    };
    let mut out = Vec::new();
    match stmt.kind() {
        "import_statement" => {
            for c in kids(stmt) {
                let mut u = blank(Kind::Straight, String::new());
                match c.kind() {
                    "dotted_name" => u.module = text(c, src).split_whitespace().collect(),
                    "aliased_import" => {
                        u.module = text(c.child_by_field_name("name")?, src).split_whitespace().collect();
                        u.alias = Some(text(c.child_by_field_name("alias")?, src).to_owned());
                    }
                    _ => return None,
                }
                out.push(u);
            }
        }
        "import_from_statement" | "future_import_statement" => {
            let module = if stmt.kind() == "future_import_statement" {
                "__future__".to_owned()
            } else {
                text(stmt.child_by_field_name("module_name")?, src).split_whitespace().collect()
            };
            let module_node = stmt.child_by_field_name("module_name");
            let mut plain = blank(Kind::From, module.clone());
            let raw = text(stmt, src);
            plain.vertical = raw.contains('(') && raw.contains('\n') && raw.trim_end().trim_end_matches(')').trim_end().ends_with(',');
            let mut aliased = Vec::new();
            for c in kids(stmt) {
                if Some(c) == module_node {
                    continue;
                }
                match c.kind() {
                    "dotted_name" => plain.members.push((text(c, src).to_owned(), None)),
                    "aliased_import" => {
                        let mut u = blank(Kind::From, module.clone());
                        u.members.push((text(c.child_by_field_name("name")?, src).to_owned(), Some(text(c.child_by_field_name("alias")?, src).to_owned())));
                        aliased.push(u);
                    }
                    "wildcard_import" => plain.star = true,
                    "comment" => {}
                    _ => return None,
                }
            }
            if !plain.members.is_empty() || plain.star {
                out.push(plain);
            }
            out.extend(aliased);
        }
        _ => return None,
    }
    if let Some(first) = out.first_mut() {
        first.leading = leading;
    }
    if let Some(last) = out.last_mut() {
        last.trailing = trailing;
    }
    Some(out)
}

fn render(u: &Unit, indent: &str, line_length: usize) -> String {
    let mut s = String::new();
    for c in &u.leading {
        s.push_str(indent);
        s.push_str(c);
        s.push('\n');
    }
    s.push_str(indent);
    match u.kind {
        Kind::Straight => {
            s.push_str("import ");
            s.push_str(&u.module);
            if let Some(a) = &u.alias {
                s.push_str(" as ");
                s.push_str(a);
            }
        }
        Kind::From => {
            let names: Vec<String> = if u.star {
                vec!["*".into()]
            } else {
                u.members.iter().map(|(n, a)| match a {
                    Some(a) => format!("{n} as {a}"),
                    None => n.clone(),
                }).collect()
            };
            let one = format!("from {} import {}", u.module, names.join(", "));
            let trailing_len = u.trailing.as_ref().map(|t| t.len() + 2).unwrap_or(0);
            if (u.vertical || indent.len() + one.len() + trailing_len > line_length) && !u.star && !names.is_empty() {
                s.push_str(&format!("from {} import (\n", u.module));
                for n in &names {
                    s.push_str(&format!("{indent}    {n},\n"));
                }
                s.push_str(indent);
                s.push(')');
            } else {
                s.push_str(&one);
            }
        }
    }
    if let Some(t) = &u.trailing {
        s.push_str("  ");
        s.push_str(t);
    }
    s
}

fn is_import(n: Node<'_>) -> bool {
    matches!(n.kind(), "import_statement" | "import_from_statement" | "future_import_statement")
}

fn line_start(src: &str, at: usize) -> usize {
    src[..at].rfind('\n').map(|i| i + 1).unwrap_or(0)
}

fn line_end(src: &str, at: usize) -> usize {
    src[at..].find('\n').map(|i| at + i).unwrap_or(src.len())
}

fn sort_block(block: &[Node<'_>], ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    let first = block[0];
    let last = *block.last().expect("non-empty");
    let indent = indent_at(src, first.start_byte());
    let mut units: Vec<Unit> = Vec::new();
    let mut readable = true;
    for (i, stmt) in block.iter().enumerate() {
        if !crate::python_util::starts_line(src, stmt.start_byte()) {
            return;
        }
        let end = stmt.end_byte();
        let le = line_end(src, end);
        let rest = src[end..le].trim();
        if !rest.is_empty() && !rest.starts_with('#') {
            return;
        }
        if text(*stmt, src).contains('\\') {
            return;
        }
        let trailing = (!rest.is_empty()).then(|| rest.to_owned());
        // Own-line comments directly above the statement.
        let mut leading = Vec::new();
        let mut prev = stmt.prev_named_sibling();
        let mut cursor = stmt.start_byte();
        while let Some(c) = prev.filter(|c| c.kind() == "comment") {
            let ls = line_start(src, c.start_byte());
            if !src[ls..c.start_byte()].trim().is_empty() || src[c.end_byte()..cursor].matches('\n').count() != 1 {
                break;
            }
            if i == 0 {
                break;
            }
            leading.insert(0, text(c, src).to_owned());
            cursor = c.start_byte();
            prev = c.prev_named_sibling();
        }
        match units_of(*stmt, src, leading, trailing) {
            Some(u) => units.extend(u),
            None => readable = false,
        }
    }
    if !readable {
        return;
    }
    // A comment in the block that belongs to no statement would be lost.
    let attached: usize = units.iter().map(|u| u.leading.len()).sum();
    let mut loose = 0;
    let mut c = first.next_named_sibling();
    while let Some(x) = c.filter(|x| x.start_byte() < last.end_byte()) {
        if x.kind() == "comment" && crate::python_util::starts_line(src, x.start_byte()) {
            loose += 1;
        }
        c = x.next_named_sibling();
    }
    let stray_comments = loose != attached;
    let stdlib: BTreeSet<String> = match ctx.data.get("python/stdlib").map(|v| v.as_ref()) {
        Some(serde_json::Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect(),
        _ => BTreeSet::new(),
    };
    let first_party: BTreeSet<String> = ctx
        .options
        .get("known_first_party")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    let line_length = ctx.options.get("line_length").and_then(|v| v.as_u64()).unwrap_or(88) as usize;

    // Canonical form: members sorted, plain `from` imports of one module merged.
    let mut canon: Vec<Unit> = Vec::new();
    for u in &units {
        let mut u = u.clone();
        u.members.sort_by(|a, b| member_cmp(&a.0, &b.0).then_with(|| natural_cmp(a.1.as_deref().unwrap_or(""), b.1.as_deref().unwrap_or(""))));
        u.members.dedup();
        let mergeable = u.kind == Kind::From && !u.star && u.members.iter().all(|m| m.1.is_none());
        if mergeable
            && let Some(prev) = canon.iter_mut().find(|p| {
                p.kind == Kind::From && p.module == u.module && !p.star && p.members.iter().all(|m| m.1.is_none()) && !p.members.is_empty()
            })
        {
            prev.members.extend(u.members.clone());
            prev.members.sort_by(|a, b| member_cmp(&a.0, &b.0));
            prev.members.dedup();
            prev.leading.extend(u.leading.clone());
            prev.vertical |= u.vertical;
            if prev.trailing.is_none() {
                prev.trailing = u.trailing.clone();
            }
            prev.inner_comments |= u.inner_comments;
            continue;
        }
        canon.push(u);
    }
    canon.sort_by(|a, b| section(a, &stdlib, &first_party).cmp(&section(b, &stdlib, &first_party)).then_with(|| unit_cmp(a, b)));
    let unchanged = canon.len() == units.len() && canon.iter().zip(&units).all(|(a, b)| a.same(b));
    if unchanged {
        return;
    }
    let lossy = stray_comments
        || units.iter().any(|u| u.inner_comments) || units.iter().filter(|u| u.trailing.is_some()).count() != canon.iter().filter(|u| u.trailing.is_some()).count();
    let mut edits = Vec::new();
    if !lossy {
        let mut rendered = String::new();
        let mut prev_section = None;
        for u in &canon {
            let s = section(u, &stdlib, &first_party);
            if let Some(p) = prev_section {
                rendered.push('\n');
                if p != s {
                    rendered.push('\n');
                }
            }
            prev_section = Some(s);
            rendered.push_str(&render(u, &indent, line_length));
        }
        // Replace from the first statement's leading comments to the end of the last line.
        let first_leading = units.first().map(|u| u.leading.len()).unwrap_or(0);
        let mut start = line_start(src, first.start_byte());
        for _ in 0..first_leading {
            start = line_start(src, start.saturating_sub(1));
        }
        let end = line_end(src, last.end_byte());
        edits.push(TextEdit::new(start, end, rendered));
    }
    out.push(NativeFinding {
        start: first.start_byte(),
        end: last.end_byte(),
        message: "Imports are not sorted: sections (future, standard library, third party, first party, local), then module, then name".into(),
        edits,
    });
}

fn unsorted_imports(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if n.kind() == "module" && (ctx.source.contains("isort: skip_file") || ctx.source.contains("isort: off")) {
        return;
    }
    if n.kind() == "block" && ctx.source.contains("isort: off") {
        return;
    }
    let mut run: Vec<Node> = Vec::new();
    for s in statements(n) {
        if is_import(s) && !text(s, ctx.source).contains("isort: skip") && !line_has_skip(s, ctx.source) {
            run.push(s);
            continue;
        }
        if !run.is_empty() {
            sort_block(&run, ctx, out);
            run.clear();
        }
    }
    if !run.is_empty() {
        sort_block(&run, ctx, out);
    }
}

fn line_has_skip(s: Node<'_>, src: &str) -> bool {
    let le = line_end(src, s.end_byte());
    src[s.end_byte()..le].contains("isort: skip") || src[s.end_byte()..le].contains("isort:skip")
}
py_rule!(UnsortedImports, "py/unsorted-imports", ["module", "block"], false, unsorted_imports);

fn all_sorted(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    let strings = dunder_all_strings(root, src);
    let mut lists: Vec<Node> = Vec::new();
    for s in strings {
        if let Some(p) = s.parent()
            && !lists.iter().any(|l| l.id() == p.id())
        {
            lists.push(p);
        }
    }
    for list in lists {
        if !matches!(list.kind(), "list" | "tuple" | "expression_list") {
            continue;
        }
        let items = kids(list);
        let has_comments = crate::util::named_children(list).iter().any(|c| c.kind() == "comment");
        let values: Option<Vec<String>> = items.iter().map(|i| if i.kind() == "string" { string_value(*i, src) } else { None }).collect();
        let Some(values) = values else { continue };
        if values.len() < 2 {
            continue;
        }
        let mut order: Vec<usize> = (0..values.len()).collect();
        // Within each group the order is natural and case-sensitive.
        order.sort_by(|&a, &b| member_rank(&values[a]).cmp(&member_rank(&values[b])).then_with(|| natural(&values[a], &values[b])));
        if order.iter().enumerate().all(|(i, &o)| i == o) {
            continue;
        }
        let mut edits = Vec::new();
        if !has_comments {
            let first = items[0];
            let last = *items.last().expect("two or more");
            let multiline = text(list, src).contains('\n');
            let column = if crate::python_util::starts_line(src, first.start_byte()) {
                indent_at(src, first.start_byte())
            } else {
                " ".repeat(src[line_start(src, first.start_byte())..first.start_byte()].chars().count())
            };
            let sep = if multiline { format!(",\n{column}") } else { ", ".to_owned() };
            let joined = order.iter().map(|&o| text(items[o], src)).collect::<Vec<_>>().join(&sep);
            edits.push(TextEdit::new(first.start_byte(), last.end_byte(), joined));
        }
        out.push(hit_fix(list, "`__all__` is not sorted (CONSTANTS, then Classes, then other names)", edits));
    }
}
py_rule!(AllSorted, "py/all-sorted", ["module"], false, all_sorted);

#[cfg(test)]
mod tests {
    use crate::{NativeContext, NativeRule};
    use codelaw_core::Language;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn fix(rule: &dyn NativeRule, src: &str) -> String {
        let root = codelaw_lang::parse(Language::Python, src);
        let mut data = BTreeMap::new();
        data.insert("python/stdlib".to_owned(), Arc::new(serde_json::json!(["os", "sys", "json", "collections", "typing", "__future__"])));
        let options = BTreeMap::new();
        let ctx = NativeContext { source: src, language: Language::Python, semantic: None, options: &options, data: &data };
        let mut out = Vec::new();
        for node in root.root().dfs() {
            if rule.kinds().contains(&node.kind().as_ref()) {
                rule.check(node, &ctx, &mut out);
            }
        }
        let mut s = src.to_owned();
        let mut edits: Vec<codelaw_core::TextEdit> = out.into_iter().flat_map(|f| f.edits).collect();
        edits.sort_by_key(|e| std::cmp::Reverse(e.start));
        for e in edits {
            s.replace_range(e.start..e.end, &e.text);
        }
        s
    }

    #[test]
    fn sorts_like_isort() {
        let src = "\"\"\"Doc.\"\"\"\nfrom __future__ import annotations\nimport sys\nimport os\nfrom . import local\nfrom .. import parent\nimport requests\nfrom collections import OrderedDict, abc, Counter, ABC_X, defaultdict as dd\nimport os.path as osp\nfrom typing import (\n    List,\n    Dict,\n)\n# comment about json\nimport json\nimport Zebra\nimport apple\nx = 1\n";
        let want = "\"\"\"Doc.\"\"\"\nfrom __future__ import annotations\n\n# comment about json\nimport json\nimport os\nimport os.path as osp\nimport sys\nfrom collections import ABC_X, Counter, OrderedDict, abc\nfrom collections import defaultdict as dd\nfrom typing import (\n    Dict,\n    List,\n)\n\nimport apple\nimport requests\nimport Zebra\n\nfrom .. import parent\nfrom . import local\nx = 1\n";
        assert_eq!(fix(&super::UnsortedImports, src), want);
        assert_eq!(fix(&super::UnsortedImports, want), want);
    }

    #[test]
    fn all_sorted() {
        assert_eq!(fix(&super::AllSorted, "__all__ = ['b', 'A', 'CONST', 'a10', 'a2']\n"), "__all__ = ['CONST', 'A', 'a2', 'a10', 'b']\n");
    }
}
