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
use codelaw_semantic::{ScopeKind, SemanticModel, SymbolKind};
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

/// `spell/known-typo-identifier`: a known typo in a name where it is declared
/// — a variable, a function, a class, a parameter, a type, a member — in
/// JavaScript, TypeScript and Python, whose semantic model resolves every
/// use (decision 5). The name splits at case changes, `_`, `-`, `.` and
/// digits; each part is looked up as a word.
///
/// The fix renames the declaration and every use, and only when that is
/// safe without reading another file: the name is not exported, not a
/// module-level name of a script or a Python module (another file may read
/// it), not a Python parameter (a caller may pass it by keyword), not a
/// class attribute or member (read through `.`), not in a shorthand
/// property (`{ recieve }` names a key too), every place the name is
/// written in the file is a use the model resolved to it, and the new name
/// is free. Otherwise the finding has no edit and its steps are manual.
pub struct KnownTypoIdentifier;

const NAME_LANGUAGES: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx, Language::Python];

/// Members: declared on a class, an interface or a type, read through `.`.
const MEMBERS: &[&str] = &["method_definition", "field_definition", "public_field_definition", "property_signature", "method_signature", "abstract_method_signature"];

impl NativeRule for KnownTypoIdentifier {
    fn id(&self) -> &'static str {
        "spell/known-typo-identifier"
    }
    fn languages(&self) -> &'static [Language] {
        NAME_LANGUAGES
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["program", "module"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(table) = typo_table(ctx) else { return };
        let Some(model) = ctx.semantic else { return };
        let root = node.get_inner_node();
        // The root only: TypeScript's `module M {}` is a `module` too.
        if root.parent().is_some() {
            return;
        }
        let src = ctx.source;
        let python = ctx.language == Language::Python;
        let mut leaves = None;
        for (id, s) in model.symbols.iter().enumerate() {
            if s.kind == SymbolKind::Import {
                continue;
            }
            let Some(&(start, end)) = s.decls.first() else { continue };
            if src.get(start..end) != Some(s.name.as_str()) {
                continue;
            }
            let typos = name_typos(table, &s.name);
            if typos.is_empty() {
                continue;
            }
            let leaves = leaves.get_or_insert_with(|| Leaves::of(root, src));
            let edits = renamed(&s.name, &typos).and_then(|new| rename_edits(model, id, root, src, python, leaves, &new, &typos)).unwrap_or_default();
            out.push(name_finding(start, end, &s.name, &typos, edits));
        }
        if python {
            return;
        }
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            if MEMBERS.contains(&n.kind())
                && let Some(name) = n.child_by_field_name("name").or_else(|| n.child_by_field_name("property"))
                && matches!(name.kind(), "property_identifier" | "private_property_identifier")
            {
                let word = text(name, src);
                let typos = name_typos(table, word);
                if !typos.is_empty() {
                    out.push(name_finding(name.start_byte(), name.end_byte(), word, &typos, Vec::new()));
                }
            }
            stack.extend(named_children(n).into_iter().rev());
        }
    }
}

/// `spell/known-typo-key`: a known typo in a name other files read — a JSON,
/// YAML or TOML key, a CSS or SCSS class, id or custom property, an HTML
/// `id` — reported once per file where it is declared, never fixed:
/// renaming it breaks what reads it.
pub struct KnownTypoKey;

impl NativeRule for KnownTypoKey {
    fn id(&self) -> &'static str {
        "spell/known-typo-key"
    }
    fn languages(&self) -> &'static [Language] {
        &[Language::Json, Language::Yaml, Language::Toml, Language::Css, Language::Scss, Language::Html]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document", "stream", "stylesheet"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        // The root only: a YAML `stream` holds `document`s.
        if node.get_inner_node().parent().is_some() {
            return;
        }
        let Some(table) = typo_table(ctx) else { return };
        let src = ctx.source;
        let mut seen = std::collections::BTreeSet::new();
        let mut stack = vec![node.get_inner_node()];
        while let Some(n) = stack.pop() {
            stack.extend(named_children(n).into_iter().rev());
            let Some((start, end)) = declared_key(n, ctx.language, src) else { continue };
            let word = &src[start..end];
            if !seen.insert(word) {
                continue;
            }
            let typos = name_typos(table, word);
            if !typos.is_empty() {
                out.push(name_finding(start, end, word, &typos, Vec::new()));
            }
        }
    }
}

/// The byte range of the name `n` declares, in a language whose names other
/// files read.
fn declared_key(n: Node<'_>, language: Language, src: &str) -> Option<(usize, usize)> {
    use Language::*;
    let inner = |n: Node<'_>| (n.start_byte() + 1, n.end_byte().saturating_sub(1).max(n.start_byte() + 1));
    match (language, n.kind()) {
        (Json, "string_content") => n.parent().filter(|p| is_key(*p)).map(|_| (n.start_byte(), n.end_byte())),
        (Yaml, "string_scalar") => in_yaml_key(n).then(|| (n.start_byte(), n.end_byte())),
        (Yaml, "double_quote_scalar" | "single_quote_scalar") => in_yaml_key(n).then(|| inner(n)),
        (Toml, "bare_key") => Some((n.start_byte(), n.end_byte())),
        (Toml, "quoted_key") => Some(inner(n)),
        (Css | Scss, "class_name" | "id_name") => Some((n.start_byte(), n.end_byte())),
        (Css | Scss, "property_name") => text(n, src).starts_with(['$', '-']).then(|| (n.start_byte(), n.end_byte())),
        (Scss, "mixin_statement" | "function_statement") => n.child_by_field_name("name").map(|c| (c.start_byte(), c.end_byte())),
        (Html, "attribute_value") => {
            let attribute = n.parent().filter(|p| p.kind() == "quoted_attribute_value").and_then(|p| p.parent()).or(n.parent())?;
            let name = named_children(attribute).into_iter().find(|c| c.kind() == "attribute_name")?;
            text(name, src).eq_ignore_ascii_case("id").then(|| (n.start_byte(), n.end_byte()))
        }
        _ => None,
    }
}

/// One part of a name that is a known typo: its offset in the name, the
/// part, and the list's corrections (`fix` or `fix1|fix2`).
type NameTypo<'a, 't> = (usize, &'a str, &'t str);

/// The parts of `name` on the known-typo list.
fn name_typos<'a, 't>(table: &'t str, name: &'a str) -> Vec<NameTypo<'a, 't>> {
    name_parts(name).into_iter().filter(|(_, p)| p.len() >= 4).filter_map(|(at, part)| lookup(table, &part.to_ascii_lowercase()).map(|f| (at, part, f))).collect()
}

/// The words of an identifier: runs of ASCII letters, split where a lower-
/// case letter meets an upper-case one (`recieveAll`) and before the last
/// capital of a run of capitals followed by a lower-case letter
/// (`HTTPServer` → `HTTP`, `Server`). `_`, `-`, `.`, digits and anything
/// else separate.
fn name_parts(name: &str) -> Vec<(usize, &str)> {
    let b = name.as_bytes();
    let mut out = Vec::new();
    let mut start = None;
    for i in 0..=b.len() {
        let letter = i < b.len() && b[i].is_ascii_alphabetic();
        match (start, letter) {
            (None, true) => start = Some(i),
            (Some(s), true) => {
                let split = (b[i - 1].is_ascii_lowercase() && b[i].is_ascii_uppercase())
                    || (b[i - 1].is_ascii_uppercase() && b[i].is_ascii_uppercase() && b.get(i + 1).is_some_and(u8::is_ascii_lowercase));
                if split {
                    out.push((s, &name[s..i]));
                    start = Some(i);
                }
            }
            (Some(s), false) => {
                out.push((s, &name[s..i]));
                start = None;
            }
            (None, false) => {}
        }
    }
    out
}

/// `name` with every typo replaced by its one correction, in the part's
/// case; `None` when a part has several corrections or a mixed case.
fn renamed(name: &str, typos: &[NameTypo<'_, '_>]) -> Option<String> {
    let mut out = name.to_owned();
    for &(at, part, fixes) in typos.iter().rev() {
        if fixes.contains('|') {
            return None;
        }
        out.replace_range(at..at + part.len(), &same_case(part, fixes)?);
    }
    Some(out)
}

fn name_finding(start: usize, end: usize, name: &str, typos: &[NameTypo<'_, '_>], edits: Vec<TextEdit>) -> NativeFinding {
    let parts: Vec<String> = typos
        .iter()
        .map(|&(_, part, fixes)| {
            let fixes: Vec<String> = fixes.split('|').map(|f| format!("`{}`", same_case(part, f).unwrap_or_else(|| f.to_owned()))).collect();
            format!("`{part}` (a known typo of {})", fixes.join(" or "))
        })
        .collect();
    NativeFinding { start, end, message: format!("`{name}` holds {}", parts.join(", ")), edits }
}

/// The identifier-like leaves and the string texts of a file, to prove a
/// rename reaches every place a name is written.
struct Leaves {
    /// Identifier leaves: start offset by text.
    names: std::collections::BTreeMap<String, Vec<usize>>,
    /// String contents: (start, end).
    strings: Vec<(usize, usize)>,
}

impl Leaves {
    fn of(root: Node<'_>, src: &str) -> Self {
        let mut names: std::collections::BTreeMap<String, Vec<usize>> = std::collections::BTreeMap::new();
        let mut strings = Vec::new();
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            let kind = n.kind();
            if kind.ends_with("identifier") && n.child_count() == 0 {
                names.entry(text(n, src).to_owned()).or_default().push(n.start_byte());
            } else if matches!(kind, "string_fragment" | "string_content") {
                strings.push((n.start_byte(), n.end_byte()));
            }
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        Leaves { names, strings }
    }

    /// Whether `name` is written in a string as a whole word.
    fn in_strings(&self, src: &str, name: &str) -> bool {
        let ident = |c: char| c.is_alphanumeric() || c == '_' || c == '$';
        self.strings.iter().any(|&(s, e)| {
            let t = &src[s..e];
            t.match_indices(name).any(|(i, _)| !t[..i].ends_with(ident) && !t[i + name.len()..].starts_with(ident))
        })
    }
}

/// Words a rename may not produce: a keyword is not a name.
const KEYWORDS: &[&str] = &[
    "await", "break", "case", "catch", "class", "const", "continue", "debugger", "default", "delete", "else", "enum", "export", "extends", "false", "finally",
    "for", "function", "import", "instanceof", "interface", "let", "new", "null", "package", "private", "protected", "public", "return", "static", "super",
    "switch", "this", "throw", "true", "typeof", "void", "while", "with", "yield", "None", "True", "False", "assert", "async", "def", "elif", "except",
    "from", "global", "lambda", "nonlocal", "pass", "raise", "while",
];

/// The edits that rename symbol `id` to `new` across the file, or `None`
/// when that is not provably safe from this file alone.
#[allow(clippy::too_many_arguments)]
fn rename_edits(
    model: &SemanticModel,
    id: usize,
    root: Node<'_>,
    src: &str,
    python: bool,
    leaves: &Leaves,
    new: &str,
    typos: &[NameTypo<'_, '_>],
) -> Option<Vec<TextEdit>> {
    let s = model.symbol(id);
    let scope = model.scopes[s.scope].kind;
    if s.exported || s.implicitly_used || scope == ScopeKind::Class || (python && s.kind == SymbolKind::Param) {
        return None;
    }
    // Another file reads a Python module's names, and a script's globals.
    if scope == ScopeKind::Module && (python || (model.imports.is_empty() && model.exports.is_empty())) {
        return None;
    }
    if KEYWORDS.contains(&new) || leaves.names.contains_key(new) || leaves.in_strings(src, &s.name) {
        return None;
    }
    let mut ranges: Vec<(usize, usize)> = s.decls.clone();
    ranges.extend(s.refs.iter().map(|&r| (model.references[r].start, model.references[r].end)));
    let mut closing = Vec::new();
    for &(start, end) in &ranges {
        let n = root.descendant_for_byte_range(start, end)?;
        if n.start_byte() != start || n.end_byte() != end || n.kind().starts_with("shorthand_property") || crate::util::in_ambient(n) {
            return None;
        }
        // `<Recieve>…</Recieve>`: the model resolves the opening name only.
        if let Some(open) = n.parent().filter(|p| p.kind() == "jsx_opening_element")
            && let Some(close) = open.parent().and_then(|e| named_children(e).into_iter().find(|c| c.kind() == "jsx_closing_element"))
            && let Some(name) = close.child_by_field_name("name")
        {
            closing.push((name.start_byte(), name.end_byte()));
        }
    }
    ranges.extend(closing);
    ranges.sort_unstable();
    ranges.dedup();
    if ranges.iter().any(|&(a, b)| src.get(a..b) != Some(s.name.as_str())) {
        return None;
    }
    // Every place the name is written is one of them.
    let written = leaves.names.get(&s.name).map_or(&[][..], Vec::as_slice);
    if written.len() != ranges.len() || written.iter().any(|w| ranges.binary_search_by_key(w, |r| r.0).is_err()) {
        return None;
    }
    let mut edits = Vec::new();
    for &(start, _) in &ranges {
        for &(at, part, fixes) in typos {
            edits.push(TextEdit::new(start + at, start + at + part.len(), same_case(part, fixes)?));
        }
    }
    Some(edits)
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

    /// A rule's findings over `src`, with the semantic model: the reported
    /// text and the file after its edits.
    fn run(rule: &dyn NativeRule, language: Language, src: &str) -> (Vec<String>, String) {
        let root = codelaw_lang::parse(language, src);
        let model = codelaw_semantic::build(language, &root, src);
        let mut data = BTreeMap::new();
        data.insert("spell/typos".to_owned(), Arc::new(serde_json::json!({ "entries": TABLE })));
        let options = BTreeMap::new();
        let ctx = NativeContext { source: src, language, semantic: model.as_ref(), options: &options, data: &data, file: &crate::DetachedFile::default() };
        let mut out = Vec::new();
        for node in root.root().dfs() {
            if rule.kinds().contains(&node.kind().as_ref()) {
                rule.check(node, &ctx, &mut out);
            }
        }
        let mut edits: Vec<TextEdit> = out.iter().flat_map(|f| f.edits.clone()).collect();
        edits.sort_by_key(|e| std::cmp::Reverse(e.start));
        let mut fixed = src.to_owned();
        for e in edits {
            fixed.replace_range(e.start..e.end, &e.text);
        }
        (out.iter().map(|f| src[f.start..f.end].to_owned()).collect(), fixed)
    }

    #[test]
    fn names_split_at_case_separators_and_digits() {
        let parts = |n: &'static str| name_parts(n).into_iter().map(|(_, p)| p).collect::<Vec<_>>();
        assert_eq!(parts("HTTPServer"), ["HTTP", "Server"]);
        assert_eq!(parts("recieveAll"), ["recieve", "All"]);
        assert_eq!(parts("RECIEVE_ALL2x"), ["RECIEVE", "ALL", "x"]);
        assert_eq!(parts("my-class.name"), ["my", "class", "name"]);
        assert_eq!(parts("$__x"), ["x"]);
    }

    #[test]
    fn a_file_local_name_is_renamed_everywhere() {
        let src = "export function f() {\n  const recieveAll = 1;\n  return recieveAll + 1;\n}\n";
        assert_eq!(run(&KnownTypoIdentifier, Language::Javascript, src), (vec!["recieveAll".into()], src.replace("recieveAll", "receiveAll")));
        let src = "def f():\n    lenght = 2\n    return lenght\n";
        assert_eq!(run(&KnownTypoIdentifier, Language::Python, src), (vec!["lenght".into()], src.replace("lenght", "length")));
        let src = "import x from 'x';\nconst Recieve = () => null;\nconst a = <Recieve>hi</Recieve>;\n";
        assert_eq!(run(&KnownTypoIdentifier, Language::Jsx, src).1, src.replace("Recieve", "Receive"));
        let src = "import x from 'x';\ntype AdressBook = { a: 1 };\nlet b: AdressBook = x;\n";
        assert_eq!(run(&KnownTypoIdentifier, Language::Typescript, src).1, src.replace("Adress", "Address"));
    }

    #[test]
    fn a_name_another_file_may_read_is_reported_but_not_renamed() {
        for (language, src) in [
            (Language::Javascript, "export const recieve = 1;\n"),
            (Language::Javascript, "const recieve = 1;\n"), // a script's global
            (Language::Python, "recieve = 1\n"), // a module's name
            (Language::Python, "def f(lenght):\n    return lenght\n"), // a keyword argument
            (Language::Python, "class A:\n    lenght = 1\n"),
            (Language::Javascript, "export function f() {\n  const recieve = 1;\n  return { recieve };\n}\n"), // a key too
            (Language::Javascript, "export function f(o) {\n  const recieve = 1;\n  return o.recieve + recieve;\n}\n"), // written elsewhere
            (Language::Javascript, "export function f() {\n  const recieve = 1;\n  const receive = 2;\n  return recieve + receive;\n}\n"), // taken
            (Language::Javascript, "export function f() {\n  const wich = 1;\n  return wich;\n}\n"), // two corrections
            (Language::Typescript, "class A {\n  recieve(): void {}\n  lenght = 1;\n}\ninterface B { adress: string }\n"), // members
        ] {
            let (found, fixed) = run(&KnownTypoIdentifier, language, src);
            assert!(!found.is_empty(), "{src}");
            assert_eq!(fixed, src, "{src}");
        }
        // Imports are another module's names; uses are never reported.
        assert_eq!(run(&KnownTypoIdentifier, Language::Javascript, "import { recieve } from 'x';\nrecieve();\n").0, Vec::<String>::new());
    }

    #[test]
    fn keys_classes_and_ids_are_reported_once_where_declared() {
        let found = |l, s| run(&KnownTypoKey, l, s).0;
        assert_eq!(found(Language::Json, "{\"recieve\": {\"adress\": \"lenght x\"}, \"b\": [{\"adress\": 1}]}"), ["recieve", "adress"]);
        assert_eq!(found(Language::Yaml, "recieve:\n  \"adress\": lenght\n"), ["recieve", "adress"]);
        assert_eq!(found(Language::Toml, "[recieve.adress]\n\"lenght\" = 1\n"), ["recieve", "adress", "lenght"]);
        assert_eq!(found(Language::Css, ".recieve-all #adress { --lenght-x: 1; color: red; }\n.recieve-all { }\n"), ["recieve-all", "adress", "--lenght-x"]);
        assert_eq!(found(Language::Scss, "$lenght: 1;\n@mixin adress-x { }\n"), ["$lenght", "adress-x"]);
        assert_eq!(found(Language::Html, "<p id=\"adress\" class=\"recieve\" title=\"lenght\">x</p>"), ["adress"]);
        let (_, fixed) = run(&KnownTypoKey, Language::Json, "{\"recieve\": 1}");
        assert_eq!(fixed, "{\"recieve\": 1}");
    }

}
