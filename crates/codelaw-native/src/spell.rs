//! The spell pack's rules (SR-SPELL). `spell/known-typo` reads the prose of
//! a parsed file — comments and docstrings, prose strings, Markdown and HTML
//! text, the HTML attributes people read, and the string values of JSON,
//! YAML and TOML — and reports each word on the pack's known-typo list, with
//! the correction as an `auto` fix (decision 5).
//!
//! Text is taken from the tree the engine parsed, never by a regex over the
//! file (decision 3). A string is prose only when it holds whitespace and no
//! interpolation; a key is a name, not prose. A token is skipped when it is a
//! URL, an email address or a mention, a path or a dotted name, a hash, a
//! UUID, base64, code, or an identifier (`snake_case`, `camelCase`, letters
//! and digits mixed): names are the identifier rule's, where they are
//! declared. Words shorter than four letters are never checked (decision 4).
//!
//! The list is the pack's `data/spell/typos.json`, written by
//! `scripts/spell-data.py`: one string of `typo:fix` entries sorted by byte,
//! searched in place, so loading it builds nothing (decision 9).

use crate::util::{named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use tree_sitter::Node;

pub struct KnownTypo;

impl NativeRule for KnownTypo {
    fn id(&self) -> &'static str {
        "spell/known-typo"
    }
    fn languages(&self) -> &'static [Language] {
        &Language::ALL
    }
    fn kinds(&self) -> &'static [&'static str] {
        &[
            "comment",
            "js_comment",
            "string_fragment",
            "string_content",
            "jsx_text",
            "text",
            "attribute_value",
            "inline",
            "string",
            "string_scalar",
            "double_quote_scalar",
            "single_quote_scalar",
            "block_scalar",
        ]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(table) = typo_table(ctx) else { return };
        let n = node.get_inner_node();
        let Some(kind) = prose_kind(n, ctx.language, ctx.source) else { return };
        let (start, end) = (n.start_byte(), n.end_byte());
        let segment = &ctx.source[start..end];
        let masked;
        let scan = if kind == Prose::Markdown {
            masked = mask_markdown(segment);
            masked.as_str()
        } else {
            segment
        };
        if kind == Prose::String && !scan.contains(char::is_whitespace) {
            return;
        }
        for (at, word) in words(scan) {
            let Some(fixes) = lookup(table, &word.to_ascii_lowercase()) else { continue };
            let (s, e) = (start + at, start + at + word.len());
            let options: Vec<&str> = fixes.split('|').collect();
            let mut finding = NativeFinding {
                start: s,
                end: e,
                message: format!("`{word}` is a known typo of {}", options.iter().map(|f| format!("`{}`", same_case(word, f).unwrap_or_else(|| (*f).to_owned()))).collect::<Vec<_>>().join(" or ")),
                edits: Vec::new(),
            };
            // One correction is safe to apply unattended; several are the
            // author's choice.
            if options.len() == 1
                && let Some(fixed) = same_case(word, options[0])
            {
                finding.edits.push(TextEdit::new(s, e, fixed));
            }
            out.push(finding);
        }
    }
}

/// How a node's text is read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Prose {
    /// Always prose: comments, docstrings, element text.
    Text,
    /// A string literal or value: prose only when it holds whitespace.
    String,
    /// Markdown inline text: code spans, links' targets and HTML masked.
    Markdown,
}

/// Whether `n` is prose in `language`, and how to read it.
fn prose_kind(n: Node<'_>, language: Language, src: &str) -> Option<Prose> {
    use Language::*;
    let kind = n.kind();
    let parent = n.parent();
    let parent_kind = parent.map(|p| p.kind()).unwrap_or("");
    match language {
        Javascript | Jsx | Typescript | Tsx => match kind {
            "comment" => Some(Prose::Text),
            "jsx_text" => Some(Prose::Text),
            "string_fragment" if parent_kind == "string" => Some(Prose::String),
            "string_fragment" if parent_kind == "template_string" && !has_child(parent?, "template_substitution") => Some(Prose::String),
            _ => None,
        },
        Python => match kind {
            "comment" => Some(Prose::Text),
            "string_content" if parent_kind == "string" && !has_child(parent?, "interpolation") => Some(Prose::String),
            _ => None,
        },
        Css | Scss => matches!(kind, "comment" | "js_comment").then_some(Prose::Text),
        Html => match kind {
            "comment" | "text" => Some(Prose::Text),
            "attribute_value" => {
                let attribute = parent.filter(|p| p.kind() == "quoted_attribute_value").and_then(|p| p.parent()).or(parent)?;
                let name = named_children(attribute).into_iter().find(|c| c.kind() == "attribute_name")?;
                matches!(text(name, src).to_ascii_lowercase().as_str(), "alt" | "title" | "aria-label" | "placeholder").then_some(Prose::String)
            }
            _ => None,
        },
        Json => (kind == "string_content" && parent.is_some_and(|p| !is_key(p))).then_some(Prose::String),
        Yaml => match kind {
            "comment" => Some(Prose::Text),
            "string_scalar" | "double_quote_scalar" | "single_quote_scalar" | "block_scalar" if !in_yaml_key(n) => Some(Prose::String),
            _ => None,
        },
        Toml => match kind {
            "comment" => Some(Prose::Text),
            "string" => Some(Prose::String),
            _ => None,
        },
        // The block tree's `inline` nodes; the inline grammar's region,
        // whose root is also `inline`, would read the same text again.
        Markdown => (kind == "inline" && parent.is_some()).then_some(Prose::Markdown),
    }
}

fn has_child(n: Node<'_>, kind: &str) -> bool {
    named_children(n).iter().any(|c| c.kind() == kind)
}

/// A JSON string that is a pair's key.
fn is_key(string: Node<'_>) -> bool {
    string.parent().and_then(|p| p.child_by_field_name("key")).is_some_and(|k| k.id() == string.id())
}

/// A YAML scalar inside a mapping pair's key.
fn in_yaml_key(n: Node<'_>) -> bool {
    let mut cur = n;
    while let Some(p) = cur.parent() {
        if matches!(p.kind(), "block_mapping_pair" | "flow_pair") {
            return p.child_by_field_name("key").is_some_and(|k| k.id() == cur.id());
        }
        cur = p;
    }
    false
}

/// Markdown inline text with code spans, link and image targets, autolinks
/// and inline HTML blanked to spaces, so offsets stay the same.
fn mask_markdown(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = b.to_vec();
    let blank = |from: usize, to: usize, out: &mut Vec<u8>| out[from..to].iter_mut().for_each(|c| *c = b' ');
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'`' => {
                let run = b[i..].iter().take_while(|&&c| c == b'`').count();
                let fence = &s[i..i + run];
                let close = s[i + run..].find(fence).map(|p| i + run + p + run).unwrap_or(b.len());
                blank(i, close, &mut out);
                i = close;
            }
            b']' if b.get(i + 1) == Some(&b'(') => {
                let close = s[i..].find(')').map(|p| i + p + 1).unwrap_or(b.len());
                blank(i + 1, close, &mut out);
                i = close;
            }
            b'<' => {
                let close = s[i..].find('>').map(|p| i + p + 1).unwrap_or(i + 1);
                blank(i, close, &mut out);
                i = close;
            }
            _ => i += 1,
        }
    }
    String::from_utf8(out).expect("only ASCII bytes and whole runs were blanked")
}

/// The words of a text worth checking, with their byte offsets.
fn words(s: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut at = 0;
    for token in s.split(|c: char| c.is_whitespace()) {
        let start = at;
        at += token.len() + s[at + token.len()..].chars().next().map_or(0, char::len_utf8);
        if token.is_empty() || skip_token(token) {
            continue;
        }
        let mut word_start = None;
        for (i, c) in token.char_indices().chain([(token.len(), ' ')]) {
            match (c.is_alphabetic(), word_start) {
                (true, None) => word_start = Some(i),
                (false, Some(w)) => {
                    let word = &token[w..i];
                    if word.len() >= 4 && word.is_ascii() {
                        out.push((start + w, word));
                    }
                    word_start = None;
                }
                _ => {}
            }
        }
    }
    out
}

/// A token that is not prose (decision 4).
fn skip_token(token: &str) -> bool {
    let t = token.trim_matches(|c: char| !c.is_alphanumeric());
    if t.is_empty() {
        return true;
    }
    // A URL, a mention, a colour or an issue, a variable.
    if token.contains("://") || t.starts_with("www.") || token.starts_with(['#', '@', '$']) {
        return true;
    }
    // An email address, a path, code.
    if t.contains(['@', '/', '\\', '_', '=', '{', '}', '<', '>', '$', '`', '#', '[', ']', '(', ')', '|', '~', '^', '*', ':', ';', ',']) {
        return true;
    }
    // A call: `name(`.
    if token.as_bytes().windows(2).any(|w| w[0].is_ascii_alphanumeric() && w[1] == b'(') {
        return true;
    }
    let bytes = t.as_bytes();
    // A dotted name, a file name, a version: `a.b`.
    if bytes.windows(3).any(|w| w[1] == b'.' && w[0].is_ascii_alphanumeric() && w[2].is_ascii_alphanumeric()) {
        return true;
    }
    let has_digit = bytes.iter().any(u8::is_ascii_digit);
    let has_alpha = bytes.iter().any(u8::is_ascii_alphabetic);
    // Letters and digits mixed: a hash, a UUID, base64, an id.
    if has_digit && has_alpha {
        return true;
    }
    // An identifier: a lower-case letter followed by an upper-case one.
    if bytes.windows(2).any(|w| w[0].is_ascii_lowercase() && w[1].is_ascii_uppercase()) {
        return true;
    }
    false
}

/// `fix` in the case of `word`: lower, Capitalised or UPPER; `None` for a
/// word in mixed case, which no unattended fix should guess at.
fn same_case(word: &str, fix: &str) -> Option<String> {
    if word.bytes().all(|c| c.is_ascii_lowercase()) {
        Some(fix.to_owned())
    } else if word.bytes().all(|c| c.is_ascii_uppercase()) {
        Some(fix.to_ascii_uppercase())
    } else if word.as_bytes()[0].is_ascii_uppercase() && word.bytes().skip(1).all(|c| c.is_ascii_lowercase()) {
        let mut f = fix.to_owned();
        f[..1].make_ascii_uppercase();
        Some(f)
    } else {
        None
    }
}

/// The `entries` string of the pack's typo table.
fn typo_table<'a>(ctx: &'a NativeContext<'_>) -> Option<&'a str> {
    ctx.data.get("spell/typos")?.get("entries")?.as_str()
}

/// The corrections of `word` in a table of space-separated `typo:fix`
/// entries sorted by byte: a binary search over the string itself.
fn lookup<'t>(table: &'t str, word: &str) -> Option<&'t str> {
    let b = table.as_bytes();
    let (mut lo, mut hi) = (0, b.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let start = b[lo..mid].iter().rposition(|&c| c == b' ').map_or(lo, |p| lo + p + 1);
        let end = b[start..hi].iter().position(|&c| c == b' ').map_or(hi, |p| start + p);
        let (typo, fixes) = table[start..end].split_once(':')?;
        match typo.cmp(word) {
            std::cmp::Ordering::Equal => return Some(fixes),
            std::cmp::Ordering::Less => lo = end + 1,
            std::cmp::Ordering::Greater => hi = start.saturating_sub(1).max(lo),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    const TABLE: &str = "adress:address flase:false lenght:length recieve:receive teh:the|tea untill:until wich:which|witch";

    fn check(language: Language, src: &str) -> Vec<(String, Vec<String>)> {
        let root = codelaw_lang::parse(language, src);
        let mut data = BTreeMap::new();
        data.insert("spell/typos".to_owned(), Arc::new(serde_json::json!({ "entries": TABLE })));
        let options = BTreeMap::new();
        let ctx = NativeContext { source: src, language, semantic: None, options: &options, data: &data, file: &crate::DetachedFile::default() };
        let mut out = Vec::new();
        let kinds = KnownTypo.kinds();
        for node in root.root().dfs() {
            if kinds.contains(&node.kind().as_ref()) {
                KnownTypo.check(node, &ctx, &mut out);
            }
        }
        out.iter().map(|f| (src[f.start..f.end].to_owned(), f.edits.iter().map(|e| e.text.clone()).collect())).collect()
    }

    fn found(language: Language, src: &str) -> Vec<String> {
        check(language, src).into_iter().map(|(w, _)| w).collect()
    }

    #[test]
    fn lookup_finds_every_entry_and_nothing_else() {
        for (typo, fix) in [("adress", "address"), ("flase", "false"), ("lenght", "length"), ("recieve", "receive"), ("teh", "the|tea"), ("untill", "until"), ("wich", "which|witch")] {
            assert_eq!(lookup(TABLE, typo), Some(fix), "{typo}");
        }
        for word in ["", "a", "adres", "adresss", "zzzz", "recieved", "length", "flas"] {
            assert_eq!(lookup(TABLE, word), None, "{word}");
        }
        assert_eq!(lookup("one:two", "one"), Some("two"));
        assert_eq!(lookup("", "one"), None);
    }

    #[test]
    fn lookup_over_the_shipped_table() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packs/spell/data/spell/typos.json");
        let doc: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).expect("the shipped table")).expect("JSON");
        let table = doc["entries"].as_str().expect("entries");
        let mut count = 0;
        for entry in table.split(' ') {
            let (typo, fixes) = entry.split_once(':').expect("typo:fix");
            assert_eq!(lookup(table, typo), Some(fixes), "{typo}");
            // A word one letter past the typo is not found.
            assert_eq!(lookup(table, &format!("{typo}{{")), None);
            count += 1;
        }
        assert_eq!(Some(count), doc["count"].as_u64().map(|c| c as usize));
        for word in ["receive", "the", "function", "aaaa", "zzzzzz"] {
            assert_eq!(lookup(table, word), None, "{word}");
        }
    }

    #[test]
    fn prose_in_every_language() {
        assert_eq!(found(Language::Javascript, "// we recieve it\nconst s = \"the adress here\";\nconst k = \"recieve\";\nconst t = `untill ${x} now`;\n"), ["recieve", "adress"]);
        assert_eq!(found(Language::Tsx, "const u = <p title=\"flase hope\">lenght of it</p>;\n"), ["flase", "lenght"]);
        assert_eq!(found(Language::Python, "# untill now\ndef f():\n    \"\"\"Recieve the adress.\"\"\"\n    return f\"recieve {x} it\"\n"), ["untill", "Recieve", "adress"]);
        assert_eq!(found(Language::Json, "{\"recieve it\": \"the adress here\", \"id\": \"recieve\"}\n"), ["adress"]);
        assert_eq!(found(Language::Yaml, "# untill\nrecieve: the adress here\nb: \"flase hope\"\n"), ["untill", "adress", "flase"]);
        assert_eq!(found(Language::Toml, "# untill\nkey = \"the adress here\"\nid = \"recieve\"\n"), ["untill", "adress"]);
        assert_eq!(found(Language::Css, "/* untill */\n.a { content: \"the adress\"; }\n"), ["untill"]);
        assert_eq!(found(Language::Scss, "// untill\n/* lenght */\n.a { b: c; }\n"), ["untill", "lenght"]);
        assert_eq!(found(Language::Html, "<!-- untill -->\n<img alt=\"the adress\" class=\"flase hope\"><p>lenght here</p>\n"), ["untill", "adress", "lenght"]);
        assert_eq!(found(Language::Markdown, "# Recieve\n\nThe `recieve` call, [the adress](http://x/untill) and <span title=\"flase\">lenght</span>.\n\n```js\nuntill\n```\n"), ["Recieve", "adress", "lenght"]);
    }

    #[test]
    fn tokens_that_are_not_prose_are_skipped() {
        let src = "// http://recieve.example/x a@recieve.org recieve.json recieve_it recieveIt recieve2 (recieve) recieve() 3f2a1b4c-recieve #recieve\n";
        assert_eq!(found(Language::Javascript, src), ["recieve"]);
    }

    #[test]
    fn the_fix_keeps_the_case_and_is_offered_only_for_one_correction() {
        assert_eq!(
            check(Language::Javascript, "// recieve Recieve RECIEVE RECieve wich\n"),
            [
                ("recieve".into(), vec!["receive".into()]),
                ("Recieve".into(), vec!["Receive".into()]),
                ("RECIEVE".into(), vec!["RECEIVE".into()]),
                ("RECieve".into(), vec![]),
                ("wich".into(), vec![]),
            ]
        );
        // `teh` is three letters, so never checked.
        assert!(check(Language::Javascript, "// teh\n").is_empty());
    }
}
