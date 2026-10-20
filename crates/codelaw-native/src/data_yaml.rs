//! Native rules for YAML in `@codelaw/rules-data`: duplicate keys, aliases
//! without an anchor, the explicit `---` start, and the plain scalars that
//! YAML 1.1 and YAML 1.2 read differently. The ideas come from the YAML 1.2
//! specification; no yamllint code or text is used (it is GPL-3.0).

use crate::util::{children, line_of, named_children, text, unquote};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::Node;

const YAML: &[Language] = &[Language::Yaml];

/// A scalar key's value: plain text, or a quoted string with its escapes
/// decoded. `None` for keys that aren't a single scalar.
fn key_value(key: Node<'_>, src: &str) -> Option<String> {
    let scalar = if key.kind() == "flow_node" { named_children(key).into_iter().find(|c| c.kind() != "comment")? } else { key };
    if named_children(key).iter().any(|c| matches!(c.kind(), "anchor" | "tag")) {
        // `&a key` or `!!str key`: compare what remains.
        let last = named_children(key).into_iter().last()?;
        return key_value(last, src);
    }
    let t = text(scalar, src);
    match scalar.kind() {
        "plain_scalar" => Some(t.trim().to_owned()),
        "single_quote_scalar" => Some(unquote(t).replace("''", "'")),
        "double_quote_scalar" => {
            let inner = unquote(t);
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
                    Some('x') => {
                        let h: String = it.by_ref().take(2).collect();
                        out.extend(u32::from_str_radix(&h, 16).ok().and_then(char::from_u32));
                    }
                    Some('u') => {
                        let h: String = it.by_ref().take(4).collect();
                        out.extend(u32::from_str_radix(&h, 16).ok().and_then(char::from_u32));
                    }
                    Some(o) => out.push(o),
                    None => {}
                }
            }
            Some(out)
        }
        _ => None,
    }
}

pub struct NoDuplicateKeys;

impl NativeRule for NoDuplicateKeys {
    fn id(&self) -> &'static str {
        "yaml/no-duplicate-keys"
    }
    fn languages(&self) -> &'static [Language] {
        YAML
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["block_mapping", "flow_mapping"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for pair in named_children(node.get_inner_node()) {
            if !matches!(pair.kind(), "block_mapping_pair" | "flow_pair") {
                continue;
            }
            let Some(key) = pair.child_by_field_name("key") else { continue };
            let Some(name) = key_value(key, ctx.source) else { continue };
            match seen.get(&name) {
                Some(&line) => out.push(NativeFinding {
                    start: key.start_byte(),
                    end: key.end_byte(),
                    message: format!("Key `{name}` is already in this mapping on line {line}; YAML forbids a key twice"),
                    edits: Vec::new(),
                }),
                None => {
                    seen.insert(name, line_of(ctx.source, key.start_byte()));
                }
            }
        }
    }
}

pub struct NoUndeclaredAlias;

impl NativeRule for NoUndeclaredAlias {
    fn id(&self) -> &'static str {
        "yaml/no-undeclared-alias"
    }
    fn languages(&self) -> &'static [Language] {
        YAML
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        // Anchors are scoped to one document and must come before the alias.
        let mut anchors: BTreeSet<&str> = BTreeSet::new();
        let mut stack = vec![node.get_inner_node()];
        let mut ordered = Vec::new();
        while let Some(n) = stack.pop() {
            ordered.push(n);
            let mut c = children(n);
            c.reverse();
            stack.extend(c);
        }
        for n in ordered {
            match n.kind() {
                "anchor_name" => {
                    anchors.insert(text(n, ctx.source));
                }
                "alias_name" if !anchors.contains(text(n, ctx.source)) => {
                    let alias = n.parent().unwrap_or(n);
                    out.push(NativeFinding {
                        start: alias.start_byte(),
                        end: alias.end_byte(),
                        message: format!("`*{}` names no anchor defined before it in this document", text(n, ctx.source)),
                        edits: Vec::new(),
                    });
                }
                _ => {}
            }
        }
    }
}

pub struct DocumentStart;

/// The first byte of a document that isn't a comment, a directive or blank.
fn content_start(doc: Node<'_>, src: &str) -> Option<usize> {
    let mut at = doc.start_byte();
    let end = doc.end_byte();
    while at < end {
        let line_end = src[at..end].find('\n').map(|i| at + i + 1).unwrap_or(end);
        let line = src[at..line_end].trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('%') {
            at = line_end;
            continue;
        }
        return Some(at);
    }
    None
}

impl NativeRule for DocumentStart {
    fn id(&self) -> &'static str {
        "yaml/document-start"
    }
    fn languages(&self) -> &'static [Language] {
        YAML
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["stream"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let src = ctx.source;
        for doc in named_children(node.get_inner_node()).into_iter().filter(|d| d.kind() == "document") {
            let Some(at) = content_start(doc, src) else { continue };
            let line = &src[at..src[at..].find('\n').map(|i| at + i).unwrap_or(src.len())];
            if line == "---" || line.starts_with("--- ") || line.starts_with("---\t") || line == "---\r" {
                continue;
            }
            if line.starts_with("...") {
                continue;
            }
            let end = at + line.trim_end().len();
            out.push(NativeFinding { start: at, end, message: "Start the document with `---`".into(), edits: vec![TextEdit::insert(at, "---\n")] });
        }
    }
}

/// A plain scalar that is a mapping value or a sequence item, not a key.
fn plain_value(n: Node<'_>) -> bool {
    let Some(plain) = n.parent().filter(|p| p.kind() == "plain_scalar") else { return false };
    let Some(flow) = plain.parent().filter(|p| p.kind() == "flow_node") else { return false };
    let Some(owner) = flow.parent() else { return false };
    owner.child_by_field_name("key").is_none_or(|k| k.id() != flow.id())
}

/// The words YAML 1.1 reads as booleans and YAML 1.2 reads as strings.
const YAML11_BOOLEANS: &[&str] = &["yes", "Yes", "YES", "no", "No", "NO", "on", "On", "ON", "off", "Off", "OFF"];

pub struct NoAmbiguousBooleans;

impl NativeRule for NoAmbiguousBooleans {
    fn id(&self) -> &'static str {
        "yaml/no-ambiguous-booleans"
    }
    fn languages(&self) -> &'static [Language] {
        YAML
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["string_scalar"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let word = text(n, ctx.source);
        if !YAML11_BOOLEANS.contains(&word) || !plain_value(n) {
            return;
        }
        out.push(NativeFinding {
            start: n.start_byte(),
            end: n.end_byte(),
            message: format!("`{word}` is the string \"{word}\" in YAML 1.2 but a boolean in YAML 1.1; quote it, or write `true`/`false`"),
            edits: vec![TextEdit::new(n.start_byte(), n.end_byte(), format!("\"{word}\""))],
        });
    }
}

pub struct NoAmbiguousNumbers;

/// `0755`: decimal 755 in YAML 1.2, octal (493) in YAML 1.1.
fn leading_zero_int(t: &str) -> bool {
    let digits = t.strip_prefix(['-', '+']).unwrap_or(t);
    digits.len() > 1 && digits.starts_with('0') && digits.bytes().all(|b| b.is_ascii_digit() || b == b'_')
}

/// `22:22` or `1:20:30.5`: a string in YAML 1.2, a base-60 number in YAML 1.1.
fn sexagesimal(t: &str) -> bool {
    let body = t.strip_prefix(['-', '+']).unwrap_or(t);
    let (ints, frac) = match body.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (body, None),
    };
    if frac.is_some_and(|f| !f.bytes().all(|b| b.is_ascii_digit() || b == b'_')) {
        return false;
    }
    let mut parts = ints.split(':');
    let Some(head) = parts.next() else { return false };
    let rest: Vec<&str> = parts.collect();
    if rest.is_empty() || head.is_empty() || !head.bytes().all(|b| b.is_ascii_digit() || b == b'_') || !head.as_bytes()[0].is_ascii_digit() {
        return false;
    }
    // YAML 1.1's integer form starts with 1-9; its float form allows 0.
    if frac.is_none() && head.starts_with('0') {
        return false;
    }
    rest.iter().all(|p| !p.is_empty() && p.len() <= 2 && p.bytes().all(|b| b.is_ascii_digit()) && p.parse::<u32>().is_ok_and(|v| v < 60))
}

impl NativeRule for NoAmbiguousNumbers {
    fn id(&self) -> &'static str {
        "yaml/no-ambiguous-numbers"
    }
    fn languages(&self) -> &'static [Language] {
        YAML
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["integer_scalar", "string_scalar"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let t = text(n, ctx.source);
        if !plain_value(n) {
            return;
        }
        let message = if n.kind() == "integer_scalar" && leading_zero_int(t) {
            format!("`{t}` is decimal in YAML 1.2 but octal in YAML 1.1; write `0o{}` for octal, drop the leading zero for decimal, or quote it", t.trim_start_matches(['-', '+']).trim_start_matches('0'))
        } else if n.kind() == "string_scalar" && sexagesimal(t) {
            format!("`{t}` is a string in YAML 1.2 but a base-60 number in YAML 1.1; quote it")
        } else {
            return;
        };
        out.push(NativeFinding { start: n.start_byte(), end: n.end_byte(), message, edits: Vec::new() });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(rule: &dyn NativeRule, src: &str) -> Vec<String> {
        crate::run(rule, Language::Yaml, src).into_iter().map(|f| src[f.start..f.end].to_owned()).collect()
    }

    #[test]
    fn duplicate_keys() {
        assert_eq!(found(&NoDuplicateKeys, "a: 1\n'a': 2\nb: {c: 1, \"c\": 2}\nd:\n  a: 1\n"), ["'a'", "\"c\""]);
        // W-17 row 29: quoted keys ending in a multi-byte character, closed
        // or not, never panic.
        assert_eq!(found(&NoDuplicateKeys, "'é': 1\n\"é\": 2\n"), ["\"é\""]);
        let _ = found(&NoDuplicateKeys, "'aé: 1\n\"bé: 2\n");
    }

    #[test]
    fn aliases() {
        assert_eq!(found(&NoUndeclaredAlias, "a: *x\nb: &x 1\nc: *x\n---\nd: *x\n"), ["*x", "*x"]);
    }

    #[test]
    fn document_start() {
        assert_eq!(found(&DocumentStart, "# c\na: 1\n"), ["a: 1"]);
        assert!(found(&DocumentStart, "# c\n---\na: 1\n---\nb: 2\n").is_empty());
        assert!(found(&DocumentStart, "%YAML 1.2\n---\na: 1\n").is_empty());
    }

    #[test]
    fn ambiguous_scalars() {
        assert_eq!(found(&NoAmbiguousBooleans, "on:\n  push:\na: yes\nb: [on, 'off', y]\nc: true\n"), ["yes", "on"]);
        assert_eq!(found(&NoAmbiguousNumbers, "a: 0755\nb: 22:22\nc: 0o14\nd: 0\ne: 1:20:30.5\nf: 10:75\ng: 0:30\nh: 12:30:00\n"), ["0755", "22:22", "1:20:30.5", "12:30:00"]);
    }
}
