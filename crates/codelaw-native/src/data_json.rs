//! Native rules for JSON in `@codelaw/rules-data`: duplicate keys, and the
//! `package.json` and `tsconfig.json` starter rules, which read one key path
//! of the top-level object and offer edits a pattern can't build.

use crate::util::{line_of, named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use std::collections::BTreeMap;
use tree_sitter::Node;

const JSON: &[Language] = &[Language::Json];

/// A JSON string's value: the text between the quotes with its escapes
/// decoded, so `"a"` and `"\u0061"` are one key.
pub(crate) fn string_value(n: Node<'_>, src: &str) -> String {
    let t = text(n, src);
    let inner = if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') { &t[1..t.len() - 1] } else { t };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
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
                let hex: String = (0..4).filter_map(|_| chars.next()).collect();
                let Ok(v) = u32::from_str_radix(&hex, 16) else {
                    out.push_str(&hex);
                    continue;
                };
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

/// The pairs of an object: decoded key, key node, value node.
pub(crate) fn pairs<'t>(obj: Node<'t>, src: &str) -> Vec<(String, Node<'t>, Option<Node<'t>>)> {
    named_children(obj)
        .into_iter()
        .filter(|p| p.kind() == "pair")
        .filter_map(|p| {
            let key = p.child_by_field_name("key")?;
            (key.kind() == "string").then(|| (string_value(key, src), key, p.child_by_field_name("value")))
        })
        .collect()
}

fn top_object(doc: Node<'_>) -> Option<Node<'_>> {
    named_children(doc).into_iter().find(|n| n.kind() == "object")
}

fn member<'t>(obj: Node<'t>, src: &str, key: &str) -> Option<(Node<'t>, Option<Node<'t>>)> {
    pairs(obj, src).into_iter().find(|(k, _, _)| k == key).map(|(_, k, v)| (k, v))
}

/// The whitespace a line starts with.
pub(crate) fn indent_of(src: &str, offset: usize) -> &str {
    let start = src[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = &src[start..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

/// The indentation one level in: the first member's, or the object's own
/// plus two spaces.
fn member_indent(obj: Node<'_>, src: &str) -> String {
    match named_children(obj).into_iter().find(|n| n.kind() != "comment") {
        Some(first) if line_of(src, first.start_byte()) != line_of(src, obj.start_byte()) => indent_of(src, first.start_byte()).to_owned(),
        _ => format!("{}  ", indent_of(src, obj.start_byte())),
    }
}

/// An edit inserting `"key": value` as the first member of `obj`, just after
/// its `{`, so nothing else moves.
fn insert_first(obj: Node<'_>, src: &str, member_text: &str) -> TextEdit {
    let at = obj.start_byte() + 1;
    let indent = member_indent(obj, src);
    let empty = !named_children(obj).iter().any(|n| n.kind() == "pair");
    if empty {
        let close = indent_of(src, obj.start_byte());
        // `{}` becomes a one-member object; anything between the braces stays.
        let inner = &src[at..obj.end_byte() - 1];
        if inner.trim().is_empty() {
            return TextEdit::new(at, obj.end_byte() - 1, format!("\n{indent}{member_text}\n{close}"));
        }
        return TextEdit::insert(at, format!("\n{indent}{member_text}"));
    }
    TextEdit::insert(at, format!("\n{indent}{member_text},"))
}

pub struct NoDuplicateKeys;

impl NativeRule for NoDuplicateKeys {
    fn id(&self) -> &'static str {
        "json/no-duplicate-keys"
    }
    fn languages(&self) -> &'static [Language] {
        JSON
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["object"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for (name, key, _) in pairs(node.get_inner_node(), ctx.source) {
            match seen.get(&name) {
                Some(&first) => out.push(NativeFinding {
                    start: key.start_byte(),
                    end: key.end_byte(),
                    message: format!("Key `{name}` is already set on line {first} of this object; most parsers keep only the last value"),
                    edits: Vec::new(),
                }),
                None => {
                    seen.insert(name, line_of(ctx.source, key.start_byte()));
                }
            }
        }
    }
}

/// Runners that read the order of `scripts` (`run-s 'build:*'` runs matches in
/// object order), so sorting would change what they do.
const ORDER_SENSITIVE: &[&str] = &["npm-run-all", "run-s ", "run-p ", "run-s\t", "run-p\t"];

/// Where a script sorts: by the name it hooks into, with `pre<name>` before and
/// `post<name>` after `<name>` when `<name>` is itself a script.
fn script_sort_key(name: &str, all: &[String]) -> (String, u8) {
    for (prefix, rank) in [("pre", 0u8), ("post", 2u8)] {
        if let Some(base) = name.strip_prefix(prefix)
            && !base.is_empty()
            && all.iter().any(|n| n == base)
        {
            return (base.to_owned(), rank);
        }
    }
    (name.to_owned(), 1)
}

pub struct PackageScriptsSorted;

impl NativeRule for PackageScriptsSorted {
    fn id(&self) -> &'static str {
        "json/package-scripts-sorted"
    }
    fn languages(&self) -> &'static [Language] {
        JSON
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let src = ctx.source;
        let Some(top) = top_object(node.get_inner_node()) else { return };
        let Some((key, Some(scripts))) = member(top, src, "scripts") else { return };
        if scripts.kind() != "object" {
            return;
        }
        let entries = pairs(scripts, src);
        let order_sensitive = entries.iter().any(|(_, _, v)| v.is_some_and(|v| ORDER_SENSITIVE.iter().any(|r| text(v, src).contains(r))));
        if order_sensitive || entries.len() < 2 {
            return;
        }
        let names: Vec<String> = entries.iter().map(|(n, _, _)| n.clone()).collect();
        let mut sorted: Vec<usize> = (0..entries.len()).collect();
        sorted.sort_by(|a, b| script_sort_key(&names[*a], &names).cmp(&script_sort_key(&names[*b], &names)).then(a.cmp(b)));
        if sorted.iter().enumerate().all(|(i, j)| i == *j) {
            return;
        }
        // Swap whole members in place: the commas and line breaks between
        // them stay where they are. Comments would be left behind, so a
        // `scripts` object with comments gets no edit.
        let has_comments = named_children(scripts).iter().any(|n| n.kind() == "comment");
        let members: Vec<Node> = entries.iter().map(|(_, k, _)| k.parent().expect("a key is in a pair")).collect();
        let edits = if has_comments {
            Vec::new()
        } else {
            members
                .iter()
                .zip(&sorted)
                .filter(|(m, j)| m.id() != members[**j].id())
                .map(|(m, j)| TextEdit::new(m.start_byte(), m.end_byte(), text(members[*j], src)))
                .collect()
        };
        out.push(NativeFinding { start: key.start_byte(), end: key.end_byte(), message: "Sort the `scripts` of package.json by name".into(), edits });
    }
}

pub struct PackageEngines;

impl NativeRule for PackageEngines {
    fn id(&self) -> &'static str {
        "json/package-engines"
    }
    fn languages(&self) -> &'static [Language] {
        JSON
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let src = ctx.source;
        let Some(top) = top_object(node.get_inner_node()) else { return };
        let range = ctx.options.get("node").and_then(|v| v.as_str()).unwrap_or(">=22");
        let range_json = serde_json::to_string(range).unwrap_or_else(|_| "\">=22\"".into());
        match member(top, src, "engines") {
            Some((_, Some(engines))) if engines.kind() == "object" => {
                if member(engines, src, "node").is_some() {
                    return;
                }
                let edit = insert_first(engines, src, &format!("\"node\": {range_json}"));
                out.push(NativeFinding {
                    start: engines.start_byte(),
                    end: engines.start_byte() + 1,
                    message: "`engines` doesn't name the Node.js versions this package supports".into(),
                    edits: vec![edit],
                });
            }
            Some(_) => {}
            None => {
                let pairs: Vec<Node> = named_children(top).into_iter().filter(|n| n.kind() == "pair").collect();
                let indent = member_indent(top, src);
                let edit = match pairs.last() {
                    Some(last) => {
                        let inner = format!("{indent}  ");
                        TextEdit::insert(last.end_byte(), format!(",\n{indent}\"engines\": {{\n{inner}\"node\": {range_json}\n{indent}}}"))
                    }
                    None => insert_first(top, src, &format!("\"engines\": {{ \"node\": {range_json} }}")),
                };
                out.push(NativeFinding {
                    start: top.start_byte(),
                    end: top.start_byte() + 1,
                    message: "package.json has no `engines.node`: say which Node.js versions the package supports".into(),
                    edits: vec![edit],
                });
            }
        }
    }
}

pub struct TsconfigStrict;

impl NativeRule for TsconfigStrict {
    fn id(&self) -> &'static str {
        "json/tsconfig-strict"
    }
    fn languages(&self) -> &'static [Language] {
        JSON
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let src = ctx.source;
        let Some(top) = top_object(node.get_inner_node()) else { return };
        let extends = member(top, src, "extends").is_some();
        let message = "Turn on `\"strict\": true` in `compilerOptions`";
        match member(top, src, "compilerOptions") {
            Some((_, Some(opts))) if opts.kind() == "object" => match member(opts, src, "strict") {
                Some((_, Some(v))) if v.kind() == "false" => {
                    out.push(NativeFinding { start: v.start_byte(), end: v.end_byte(), message: message.into(), edits: vec![TextEdit::new(v.start_byte(), v.end_byte(), "true")] });
                }
                Some(_) => {}
                // A config that extends another may inherit `strict`.
                None if extends => {}
                None => out.push(NativeFinding {
                    start: opts.start_byte(),
                    end: opts.start_byte() + 1,
                    message: message.into(),
                    edits: vec![insert_first(opts, src, "\"strict\": true")],
                }),
            },
            Some(_) => {}
            None if extends => {}
            None => {
                let indent = member_indent(top, src);
                let block = format!("\"compilerOptions\": {{\n{indent}  \"strict\": true\n{indent}}}");
                out.push(NativeFinding { start: top.start_byte(), end: top.start_byte() + 1, message: message.into(), edits: vec![insert_first(top, src, &block)] });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(rule: &dyn NativeRule, src: &str) -> (Vec<String>, String) {
        let found = crate::run(rule, Language::Json, src);
        let msgs = found.iter().map(|f| f.message.clone()).collect();
        let mut edits: Vec<TextEdit> = found.into_iter().flat_map(|f| f.edits).collect();
        edits.sort_by_key(|e| std::cmp::Reverse(e.start));
        let mut out = src.to_owned();
        for e in edits {
            out.replace_range(e.start..e.end, &e.text);
        }
        (msgs, out)
    }

    #[test]
    fn duplicate_keys_decode_escapes() {
        let (msgs, _) = fixed(&NoDuplicateKeys, "{\"a\": 1, \"\\u0061\": 2, \"b\": {\"a\": 1}}");
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn scripts_sort_with_hooks() {
        let src = "{\n  \"scripts\": {\n    \"test\": \"t\",\n    \"build\": \"b\",\n    \"pretest\": \"p\",\n    \"prepare\": \"x\"\n  }\n}\n";
        let (_, out) = fixed(&PackageScriptsSorted, src);
        assert_eq!(out, "{\n  \"scripts\": {\n    \"build\": \"b\",\n    \"prepare\": \"x\",\n    \"pretest\": \"p\",\n    \"test\": \"t\"\n  }\n}\n");
        let (msgs, _) = fixed(&PackageScriptsSorted, "{\"scripts\": {\"b\": \"run-s a:*\", \"a:x\": \"\"}}");
        assert!(msgs.is_empty());
    }

    #[test]
    fn engines_inserted() {
        let (_, out) = fixed(&PackageEngines, "{\n  \"name\": \"x\"\n}\n");
        assert_eq!(out, "{\n  \"name\": \"x\",\n  \"engines\": {\n    \"node\": \">=22\"\n  }\n}\n");
        let (_, out) = fixed(&PackageEngines, "{\n  \"engines\": {\n    \"npm\": \">=9\"\n  }\n}\n");
        assert_eq!(out, "{\n  \"engines\": {\n    \"node\": \">=22\",\n    \"npm\": \">=9\"\n  }\n}\n");
    }

    #[test]
    fn tsconfig_strict() {
        let (_, out) = fixed(&TsconfigStrict, "{\n  \"compilerOptions\": {\n    \"target\": \"es2022\"\n  }\n}\n");
        assert_eq!(out, "{\n  \"compilerOptions\": {\n    \"strict\": true,\n    \"target\": \"es2022\"\n  }\n}\n");
        let (_, out) = fixed(&TsconfigStrict, "{\n  \"include\": [\"src\"]\n}\n");
        assert_eq!(out, "{\n  \"compilerOptions\": {\n    \"strict\": true\n  },\n  \"include\": [\"src\"]\n}\n");
        let (msgs, _) = fixed(&TsconfigStrict, "{\"extends\": \"./base.json\", \"compilerOptions\": {}}");
        assert!(msgs.is_empty());
    }
}
