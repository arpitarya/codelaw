//! The ignore markers of SR-FORMATTER decision 6, in every engine.
//!
//! `codelaw-format-ignore` and `prettier-ignore` skip the next node. No
//! engine knows both, so each gets them in the form it does know:
//!
//! - **a native directive** (Biome, pretty_yaml, dprint-plugin-markdown,
//!   markup_fmt): each marker comment is rewritten to the engine's own
//!   directive before formatting, and restored, in order, after;
//! - **`# fmt: off` / `# fmt: on`** (Ruff): the marker becomes `# fmt: off`
//!   and a `# fmt: on` follows the next statement; both are undone after;
//! - **a splice** (taplo, which has no directive): the file is formatted
//!   and the node after each marker is put back as it was written.
//!
//! Ruff's own `# fmt: off`/`# fmt: on`/`# fmt: skip` need none of this. A
//! marker the engine drops or moves fails the file (`format-engine`), so a
//! fenced-off node is never reformatted silently.

use crate::FormatError;
use std::ops::Range;

/// codelaw's own marker, then the one honoured for compatibility.
pub(crate) const MARKERS: [&str; 2] = ["codelaw-format-ignore", "prettier-ignore"];

/// Biome's directive (`biome-ignore format: <reason>`), with codelaw as
/// the reason.
pub(crate) const BIOME: &str = "biome-ignore format: codelaw";

/// The directive pretty_yaml, dprint-plugin-markdown and markup_fmt are
/// configured with.
pub(crate) const PRETTIER: &str = "prettier-ignore";

/// Whether `text` holds a marker at all: the fast path skips the rest.
fn any_marker(text: &str) -> bool {
    MARKERS.iter().any(|m| text.contains(m))
}

fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

/// Each of `words` that starts a comment (an opener, then only spaces or
/// tabs) and ends at a word boundary, as (offset, word), in order.
fn comment_words<'w>(text: &str, openers: &[&str], words: &[&'w str]) -> Vec<(usize, &'w str)> {
    let mut out = Vec::new();
    for (i, _) in text.char_indices() {
        let Some(word) = words.iter().find(|w| text[i..].starts_with(**w)) else { continue };
        if text[i + word.len()..].starts_with(word_char) {
            continue;
        }
        let before = text[..i].trim_end_matches([' ', '\t']);
        if openers.iter().any(|o| before.ends_with(o)) {
            out.push((i, *word));
        }
    }
    out
}

/// Rewrite every marker comment to `native`, format with `engine`, and
/// restore the markers in the order they were written.
pub(crate) fn native(text: &str, openers: &[&str], native: &str, engine: impl FnOnce(&str) -> Result<String, FormatError>) -> Result<String, FormatError> {
    if !any_marker(text) {
        return engine(text);
    }
    let mut words: Vec<&str> = MARKERS.to_vec();
    if !words.contains(&native) {
        words.push(native);
    }
    let mut rewritten = String::with_capacity(text.len());
    let mut originals = Vec::new();
    let mut at = 0;
    for (i, word) in comment_words(text, openers, &words) {
        rewritten.push_str(&text[at..i]);
        rewritten.push_str(native);
        originals.push(word);
        at = i + word.len();
    }
    rewritten.push_str(&text[at..]);
    let out = engine(&rewritten)?;
    let found = comment_words(&out, openers, &[native]);
    if found.len() != originals.len() {
        return Err(FormatError::Engine(format!("the formatter moved an ignore marker ({} written, {} after formatting)", originals.len(), found.len())));
    }
    let mut restored = String::with_capacity(out.len());
    let mut at = 0;
    for ((i, _), original) in found.into_iter().zip(originals) {
        restored.push_str(&out[at..i]);
        restored.push_str(original);
        at = i + native.len();
    }
    restored.push_str(&out[at..]);
    Ok(restored)
}

// ---- Ruff: `# fmt: off` before the next statement, `# fmt: on` after ----

/// What a Ruff pragma line in the rewritten text becomes after formatting.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Pragma {
    /// The user's own; left as it is.
    Keep,
    /// A marker comment codelaw rewrote; put back as it was written.
    Restore(String),
    /// A `# fmt: on` codelaw added; removed.
    Delete,
}

/// The own-line `# fmt: off` / `# fmt: on` lines of a text, as the byte
/// range of each line without its line break. Counted the same way before
/// and after formatting, a line inside a string included.
fn pragma_lines(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut start = 0;
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        if let Some(c) = body.trim_start().strip_prefix('#') {
            let c = c.trim();
            let cmd = c.strip_prefix("fmt:").map(str::trim);
            if matches!(cmd, Some("off" | "on")) || matches!(c.strip_prefix("yapf:").map(str::trim), Some("disable" | "enable")) {
                out.push(start..start + body.len());
            }
        }
        start += line.len();
    }
    out
}

/// The marker an own-line comment is, if it is one.
fn marker_comment(body: &str) -> Option<&'static str> {
    let c = body.trim_start().strip_prefix('#')?.trim_start();
    MARKERS.iter().copied().find(|m| c.strip_prefix(m).is_some_and(|rest| !rest.starts_with(word_char)))
}

/// Every statement of a module, in source order: (range, compound).
fn statements(text: &str) -> Option<Vec<(Range<usize>, bool)>> {
    use ruff_python_ast::Stmt;
    use ruff_python_ast::statement_visitor::{StatementVisitor, walk_stmt};
    use ruff_text_size::Ranged;
    struct Collect(Vec<(Range<usize>, bool)>);
    impl<'a> StatementVisitor<'a> for Collect {
        fn visit_stmt(&mut self, stmt: &'a Stmt) {
            let compound = matches!(
                stmt,
                Stmt::FunctionDef(_) | Stmt::ClassDef(_) | Stmt::If(_) | Stmt::For(_) | Stmt::While(_) | Stmt::With(_) | Stmt::Try(_) | Stmt::Match(_)
            );
            let r = stmt.range();
            self.0.push((usize::from(r.start())..usize::from(r.end()), compound));
            walk_stmt(self, stmt);
        }
    }
    let parsed = ruff_python_parser::parse_module(text).ok()?;
    let mut c = Collect(Vec::new());
    c.visit_body(&parsed.syntax().body);
    c.0.sort_by_key(|(r, _)| r.start);
    Some(c.0)
}

/// Ruff with codelaw's markers: each own-line marker comment followed, at
/// its own indentation, by a statement becomes `# fmt: off`, and a
/// `# fmt: on` line follows that statement. A marker inside a simple
/// statement (an expression) or with nothing after it at its level skips
/// nothing, as `prettier-ignore` before no node does.
pub(crate) fn python(text: &str, engine: impl FnOnce(&str) -> Result<String, FormatError>) -> Result<String, FormatError> {
    if !any_marker(text) {
        return engine(text);
    }
    let Some(stmts) = statements(text) else { return engine(text) };
    // (offset, replaced up to, inserted, the marker comment it stands for)
    let mut edits: Vec<(usize, usize, String, Option<String>)> = Vec::new();
    let mut covered = 0;
    let mut start = 0;
    for line in text.split_inclusive('\n') {
        let line_start = start;
        start += line.len();
        let body = line.trim_end_matches(['\n', '\r']);
        if marker_comment(body).is_none() {
            continue;
        }
        let indent = &body[..body.len() - body.trim_start().len()];
        let comment = line_start + indent.len();
        if comment < covered || stmts.iter().any(|(r, compound)| !compound && r.start < comment && comment < r.end) {
            continue;
        }
        let Some((next, _)) = stmts.iter().find(|(r, _)| r.start >= start) else { continue };
        let next_line = text[..next.start].rfind('\n').map_or(0, |i| i + 1);
        if &text[next_line..next.start] != indent {
            continue;
        }
        let on = match text[next.end..].find('\n') {
            Some(i) => (next.end + i + 1, format!("{indent}# fmt: on\n")),
            None => (text.len(), format!("\n{indent}# fmt: on\n")),
        };
        edits.push((comment, line_start + body.len(), "# fmt: off".into(), Some(body.trim_start().to_owned())));
        edits.push((on.0, on.0, on.1, None));
        covered = on.0;
    }
    if edits.is_empty() {
        return engine(text);
    }
    edits.sort_by_key(|e| e.0);
    let mut rewritten = String::with_capacity(text.len() + edits.len() * 12);
    let mut at = 0;
    for (from, to, insert, _) in &edits {
        rewritten.push_str(&text[at..*from]);
        rewritten.push_str(insert);
        at = *to;
    }
    rewritten.push_str(&text[at..]);
    // What each pragma line of the rewritten text becomes.
    let mut ours: Vec<(usize, Pragma)> = Vec::new();
    let mut shift: isize = 0;
    for (from, to, insert, marker) in &edits {
        let pos = (*from as isize + shift) as usize + insert.find('#').unwrap_or(0);
        ours.push((pos, marker.clone().map_or(Pragma::Delete, Pragma::Restore)));
        shift += insert.len() as isize - (*to - *from) as isize;
    }
    let plan: Vec<Pragma> =
        pragma_lines(&rewritten).into_iter().map(|r| ours.iter().find(|(pos, _)| r.contains(pos)).map_or(Pragma::Keep, |(_, p)| p.clone())).collect();
    let out = engine(&rewritten)?;
    let found = pragma_lines(&out);
    if found.len() != plan.len() {
        return Err(FormatError::Engine(format!("the formatter moved an ignore marker ({} pragmas written, {} after formatting)", plan.len(), found.len())));
    }
    let mut restored = String::with_capacity(out.len());
    let mut at = 0;
    for (r, p) in found.into_iter().zip(plan) {
        match p {
            Pragma::Keep => {}
            Pragma::Restore(comment) => {
                let line = &out[r.clone()];
                let indent = line.len() - line.trim_start().len();
                restored.push_str(&out[at..r.start + indent]);
                restored.push_str(&comment);
                at = r.end;
            }
            Pragma::Delete => {
                restored.push_str(&out[at..r.start]);
                at = if out[r.end..].starts_with("\r\n") {
                    r.end + 2
                } else if out[r.end..].starts_with('\n') {
                    r.end + 1
                } else {
                    r.end
                };
            }
        }
    }
    restored.push_str(&out[at..]);
    Ok(restored)
}

// ---- taplo: put the node after each marker back as written ----

/// The node each marker comment of a TOML text skips: the next entry,
/// table header or array element. `None` when nothing follows it.
fn toml_ignored(text: &str) -> Vec<Option<Range<usize>>> {
    use taplo::rowan::NodeOrToken;
    use taplo::syntax::SyntaxKind;
    let root = taplo::parser::parse(text).into_syntax();
    let range = |r: taplo::rowan::TextRange| usize::from(r.start())..usize::from(r.end());
    let mut markers = Vec::new();
    let mut nodes = Vec::new();
    for element in root.descendants_with_tokens() {
        match element {
            NodeOrToken::Token(t) if t.kind() == SyntaxKind::COMMENT && marker_comment(t.text()).is_some() => markers.push(range(t.text_range())),
            NodeOrToken::Node(n)
                if matches!(n.kind(), SyntaxKind::ENTRY | SyntaxKind::TABLE_HEADER | SyntaxKind::TABLE_ARRAY_HEADER)
                    || (n.kind() == SyntaxKind::VALUE && n.parent().is_some_and(|p| p.kind() == SyntaxKind::ARRAY)) =>
            {
                nodes.push(range(n.text_range()))
            }
            _ => {}
        }
    }
    markers.iter().map(|m| nodes.iter().find(|n| n.start >= m.end).cloned()).collect()
}

/// taplo, then the splice: each ignored node of the output is replaced by
/// the same node of the input. A node inside another ignored one is
/// already restored with it.
pub(crate) fn toml(text: &str, engine: impl FnOnce(&str) -> Result<String, FormatError>) -> Result<String, FormatError> {
    let out = engine(text)?;
    if !any_marker(text) {
        return Ok(out);
    }
    let before = toml_ignored(text);
    let after = toml_ignored(&out);
    if before.len() != after.len() {
        return Err(FormatError::Engine(format!("the formatter moved an ignore marker ({} written, {} after formatting)", before.len(), after.len())));
    }
    let mut pairs: Vec<(Range<usize>, Range<usize>)> = before.into_iter().zip(after).filter_map(|(b, a)| Some((b?, a?))).collect();
    pairs.sort_by_key(|(_, a)| a.start);
    let mut kept: Vec<(Range<usize>, Range<usize>)> = Vec::new();
    for (b, a) in pairs {
        if kept.last().is_some_and(|(_, k)| a.start < k.end) {
            continue;
        }
        kept.push((b, a));
    }
    let mut restored = out.clone();
    for (b, a) in kept.into_iter().rev() {
        restored.replace_range(a, &text[b]);
    }
    Ok(restored)
}
