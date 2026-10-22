//! Native rules for Markdown in `@codelaw/rules-data`, reimplementing the
//! documented behaviour of markdownlint's non-layout rules (MIT); no
//! markdownlint code or text is used.
//!
//! Markdown parses twice: the block grammar gives headings, code blocks,
//! tables and reference definitions; the inline grammar, run on each block
//! `inline` node, gives links, images, emphasis and code spans. A rule that
//! only needs inline nodes is dispatched on the inline grammar (its kinds
//! exist only there); a rule that needs both — fragments against headings,
//! labels against definitions — runs on the block `document` and parses the
//! inline content itself.

use crate::util::{named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::{Grammar, SyntaxNode};
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::Node;

const MD: &[Language] = &[Language::Markdown];

fn finding(start: usize, end: usize, message: impl Into<String>) -> NativeFinding {
    NativeFinding { start, end, message: message.into(), edits: Vec::new() }
}

/// Every node under `root` in document order.
fn preorder(root: Node<'_>) -> Vec<Node<'_>> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        out.push(n);
        let mut c = n.walk();
        let kids: Vec<Node> = n.children(&mut c).collect();
        stack.extend(kids.into_iter().rev());
    }
    out
}

// ---------------------------------------------------------------- headings

struct Heading<'t> {
    node: Node<'t>,
    level: usize,
    /// The heading's inline content node.
    content: Option<Node<'t>>,
    /// The `#` marker or the setext underline.
    marker: Option<Node<'t>>,
    setext: bool,
}

fn heading(n: Node<'_>) -> Option<Heading<'_>> {
    let kids = named_children(n);
    match n.kind() {
        "atx_heading" => {
            let marker = kids.iter().copied().find(|c| c.kind().starts_with("atx_h") && c.kind().ends_with("_marker"))?;
            let level = marker.kind()[5..6].parse().ok()?;
            Some(Heading { node: n, level, content: n.child_by_field_name("heading_content"), marker: Some(marker), setext: false })
        }
        "setext_heading" => {
            let marker = kids.iter().copied().find(|c| c.kind().starts_with("setext_h"))?;
            let level = if marker.kind() == "setext_h1_underline" { 1 } else { 2 };
            let content = n.child_by_field_name("heading_content").and_then(|p| named_children(p).into_iter().find(|c| c.kind() == "inline"));
            Some(Heading { node: n, level, content, marker: Some(marker), setext: true })
        }
        _ => None,
    }
}

fn headings(doc: Node<'_>) -> Vec<Heading<'_>> {
    preorder(doc).into_iter().filter_map(heading).collect()
}

/// Whether YAML front matter names a title (`title:`), which then acts as the
/// document's top-level heading.
fn front_matter_title(doc: Node<'_>, src: &str) -> bool {
    named_children(doc).into_iter().filter(|c| c.kind() == "minus_metadata").any(|m| {
        text(m, src).lines().any(|l| {
            let l = l.trim_start();
            l.strip_prefix("title").is_some_and(|r| r.trim_start().starts_with([':', '=']))
        })
    })
}

/// The inline grammar's tree of one block `inline` node.
fn parse_inline(src: &str, inline: Node<'_>) -> codelaw_lang::Root {
    codelaw_lang::parse_grammar(Grammar::MarkdownInline, &src[inline.byte_range()])
}

/// The rendered text of inline content: delimiters, link destinations and
/// HTML tags dropped, link text and code-span content kept.
fn plain_text(n: Node<'_>, src: &str, out: &mut String) {
    match n.kind() {
        "code_span" => {
            let t = text(n, src);
            let ticks = t.len() - t.trim_start_matches('`').len();
            out.push_str(t[ticks..t.len().saturating_sub(ticks).max(ticks)].trim());
            return;
        }
        "inline_link" | "full_reference_link" | "collapsed_reference_link" | "shortcut_link" => {
            if let Some(t) = named_children(n).into_iter().find(|c| c.kind() == "link_text") {
                plain_text(t, src, out);
            }
            return;
        }
        "image" | "html_tag" | "emphasis_delimiter" | "code_span_delimiter" => return,
        "backslash_escape" => {
            out.push_str(&text(n, src)[1..]);
            return;
        }
        "uri_autolink" | "email_autolink" => {
            out.push_str(text(n, src).trim_start_matches('<').trim_end_matches('>'));
            return;
        }
        _ => {}
    }
    let mut at = n.start_byte();
    let mut c = n.walk();
    for child in n.named_children(&mut c) {
        out.push_str(&src[at..child.start_byte()]);
        plain_text(child, src, out);
        at = child.end_byte();
    }
    out.push_str(&src[at..n.end_byte()]);
}

/// A heading's text as rendered, without an ATX closing sequence.
fn heading_text(h: &Heading<'_>, src: &str) -> String {
    let Some(content) = h.content else { return String::new() };
    let raw = &src[content.byte_range()];
    let mut out = String::new();
    if raw.contains(['`', '*', '_', '[', '<', '\\', '!', '~']) {
        let tree = codelaw_lang::parse_grammar(Grammar::MarkdownInline, raw);
        plain_text(tree.root().get_inner_node(), raw, &mut out);
    } else {
        out.push_str(raw);
    }
    let mut t = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if !h.setext {
        let stripped = t.trim_end_matches('#');
        if stripped.len() != t.len() && (stripped.is_empty() || stripped.ends_with(' ')) {
            t = stripped.trim_end().to_owned();
        }
    }
    t
}

pub struct HeadingIncrement;

impl NativeRule for HeadingIncrement {
    fn id(&self) -> &'static str {
        "md/heading-increment"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let doc = node.get_inner_node();
        let src = ctx.source;
        let hs = headings(doc);
        let title = front_matter_title(doc, src);
        // New levels for the whole outline: each heading nests one level
        // under the nearest earlier heading of a lower written level, so a
        // fixed heading's subsections move with it.
        let mut stack: Vec<(usize, usize)> = if title { vec![(1, 1)] } else { Vec::new() };
        let mut levels = Vec::with_capacity(hs.len());
        for h in &hs {
            while stack.last().is_some_and(|(orig, _)| *orig >= h.level) {
                stack.pop();
            }
            let new = match stack.last() {
                Some((_, parent)) => (parent + 1).min(h.level),
                None => h.level,
            };
            levels.push(new);
            stack.push((h.level, new));
        }
        let mut edits = Vec::new();
        for (h, new) in hs.iter().zip(&levels) {
            let (Some(marker), true) = (h.marker, *new != h.level) else { continue };
            let replacement = if h.setext { "=".repeat(text(marker, src).trim_end().len()) } else { "#".repeat(*new) };
            let end = if h.setext { marker.start_byte() + text(marker, src).trim_end().len() } else { marker.end_byte() };
            edits.push(TextEdit::new(marker.start_byte(), end, replacement));
        }
        let mut prev = if title { Some(1) } else { None };
        for h in &hs {
            if let Some(p) = prev
                && h.level > p + 1
            {
                let mut f = finding(h.node.start_byte(), h.content.map(|c| c.end_byte()).unwrap_or(h.node.end_byte()), format!("Heading level {} follows level {p}; go down one level at a time", h.level));
                f.edits = edits.clone();
                out.push(f);
            }
            prev = Some(h.level);
        }
    }
}

pub struct SingleH1;

impl NativeRule for SingleH1 {
    fn id(&self) -> &'static str {
        "md/single-h1"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let doc = node.get_inner_node();
        let level = ctx.options.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let mut has_title = front_matter_title(doc, ctx.source);
        for h in headings(doc).into_iter().filter(|h| h.level == level) {
            if has_title {
                let end = h.content.map(|c| c.end_byte()).unwrap_or(h.node.end_byte());
                out.push(finding(h.node.start_byte(), end, "The document already has a title; make this a lower-level heading"));
            }
            has_title = true;
        }
    }
}

pub struct NoDuplicateHeading;

impl NativeRule for NoDuplicateHeading {
    fn id(&self) -> &'static str {
        "md/no-duplicate-heading"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let siblings_only = ctx.options.get("siblings_only").and_then(|v| v.as_bool()).unwrap_or(true);
        let hs = headings(node.get_inner_node());
        // Parent of each heading in the outline, by index; `None` is the top.
        let mut stack: Vec<(usize, usize)> = Vec::new();
        let mut seen: BTreeMap<Option<usize>, BTreeSet<String>> = BTreeMap::new();
        for (i, h) in hs.iter().enumerate() {
            while stack.last().is_some_and(|(lvl, _)| *lvl >= h.level) {
                stack.pop();
            }
            let parent = if siblings_only { stack.last().map(|(_, idx)| *idx) } else { None };
            stack.push((h.level, i));
            let t = heading_text(h, ctx.source);
            if t.is_empty() {
                continue;
            }
            if !seen.entry(parent).or_default().insert(t.clone()) {
                let end = h.content.map(|c| c.end_byte()).unwrap_or(h.node.end_byte());
                let scope = if siblings_only { "under the same parent heading" } else { "in this document" };
                out.push(finding(h.node.start_byte(), end, format!("Another heading {scope} is already called \"{t}\"")));
            }
        }
    }
}

pub struct FirstLineHeading;

/// Whether an HTML block is only a comment.
fn html_comment(t: &str) -> bool {
    let t = t.trim();
    t.starts_with("<!--") && t.ends_with("-->")
}

impl NativeRule for FirstLineHeading {
    fn id(&self) -> &'static str {
        "md/first-line-heading"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let doc = node.get_inner_node();
        let src = ctx.source;
        if front_matter_title(doc, src) {
            return;
        }
        let level = ctx.options.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let preamble = ctx.options.get("allow_preamble").and_then(|v| v.as_bool()).unwrap_or(false);
        let blocks: Vec<Node> = preorder(doc)
            .into_iter()
            .filter(|n| n.parent().is_some_and(|p| matches!(p.kind(), "document" | "section")))
            .filter(|n| n.is_named() && !matches!(n.kind(), "section" | "minus_metadata" | "plus_metadata" | "link_reference_definition"))
            .filter(|n| !(n.kind() == "html_block" && html_comment(text(*n, src))))
            .filter(|n| !definition_paragraph(*n, src))
            .collect();
        let Some(first) = blocks.first() else { return };
        let is_title = |n: &Node| {
            heading(*n).is_some_and(|h| h.level == level) || (matches!(n.kind(), "html_block" | "paragraph") && text(*n, src).trim_start().to_ascii_lowercase().starts_with(&format!("<h{level}")))
        };
        if is_title(first) {
            return;
        }
        if preamble && blocks.iter().find(|n| heading(**n).is_some()).is_some_and(is_title) {
            return;
        }
        let line_end = src[first.start_byte()..].find('\n').map(|i| first.start_byte() + i).unwrap_or(first.end_byte()).min(first.end_byte());
        out.push(finding(first.start_byte(), line_end, format!("Start the document with a level-{level} heading that names it")));
    }
}

// --------------------------------------------------------- code and tables

pub struct CommandsShowOutput;

impl NativeRule for CommandsShowOutput {
    fn id(&self) -> &'static str {
        "md/commands-show-output"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["fenced_code_block", "indented_code_block"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let src = ctx.source;
        let content = if n.kind() == "fenced_code_block" {
            let Some(c) = named_children(n).into_iter().find(|c| c.kind() == "code_fence_content") else { return };
            c
        } else {
            n
        };
        let start = content.start_byte();
        let body = &src[content.byte_range()];
        let mut edits = Vec::new();
        let mut lines = 0;
        let mut offset = start;
        for line in body.split_inclusive('\n') {
            let trimmed = line.trim_start();
            if !trimmed.trim().is_empty() {
                lines += 1;
                let Some(rest) = trimmed.strip_prefix('$') else { return };
                let gap = rest.len() - rest.trim_start_matches([' ', '\t']).len();
                if gap == 0 && !rest.trim().is_empty() {
                    return;
                }
                let at = offset + (line.len() - trimmed.len());
                edits.push(TextEdit::delete(at, at + 1 + gap));
            }
            offset += line.len();
        }
        if lines == 0 {
            return;
        }
        let mut f = finding(n.start_byte(), n.start_byte() + text(n, src).find('\n').unwrap_or(0), "Every command here starts with `$` and none shows output; drop the `$` so the commands can be copied");
        f.edits = edits;
        out.push(f);
    }
}

pub struct TableColumnCount;

impl NativeRule for TableColumnCount {
    fn id(&self) -> &'static str {
        "md/table-column-count"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["pipe_table"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let rows = named_children(node.get_inner_node());
        let cells = |r: &Node| named_children(*r).iter().filter(|c| c.kind() == "pipe_table_cell").count();
        let Some(header) = rows.iter().find(|r| r.kind() == "pipe_table_header") else { return };
        let want = cells(header);
        for row in rows.iter().filter(|r| r.kind() == "pipe_table_row") {
            let got = cells(row);
            if got != want {
                let what = if got < want { "fewer" } else { "more" };
                let end = row.start_byte() + text(*row, ctx.source).trim_end().len();
                out.push(finding(row.start_byte(), end, format!("This row has {got} cells, {what} than the header's {want}")));
            }
        }
    }
}

// ------------------------------------------------------------------- HTML

pub struct NoInlineHtml;

/// The element names of the start tags in a stretch of HTML.
fn start_tags(t: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let b = t.as_bytes();
    let mut i = 0;
    while let Some(off) = t[i..].find('<') {
        let at = i + off;
        let name: String = t[at + 1..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
        if b.get(at + 1).is_some_and(|c| c.is_ascii_alphabetic()) {
            out.push((at, name));
        }
        i = at + 1;
    }
    out
}

impl NativeRule for NoInlineHtml {
    fn id(&self) -> &'static str {
        "md/no-inline-html"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["html_block", "html_tag"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let allowed: BTreeSet<String> =
            ctx.options.get("allowed_elements").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|v| v.as_str()).map(|s| s.to_ascii_lowercase()).collect();
        let t = text(n, ctx.source);
        for (at, name) in start_tags(t) {
            if n.kind() == "html_tag" && at != 0 {
                continue;
            }
            if allowed.contains(&name.to_ascii_lowercase()) {
                continue;
            }
            let start = n.start_byte() + at;
            out.push(finding(start, start + 1 + name.len(), format!("`<{name}>` is raw HTML; use Markdown, or allow the element")));
        }
    }
}

// ---------------------------------------------------------- inline content

/// Whether a node is the inline grammar's root, not the block grammar's
/// `inline` node of the same name.
fn inline_root(n: Node<'_>) -> bool {
    n.kind() == "inline" && n.parent().is_none()
}

pub struct NoReversedLinks;

impl NativeRule for NoReversedLinks {
    fn id(&self) -> &'static str {
        "md/no-reversed-links"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["shortcut_link"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let src = ctx.source;
        let target = &text(n, src)[1..text(n, src).len() - 1];
        if target.is_empty() || target.starts_with('^') {
            return;
        }
        let before = &src[..n.start_byte()];
        let Some(inner) = before.strip_suffix(')') else { return };
        let Some(open) = inner.rfind('(') else { return };
        let label = &inner[open + 1..];
        if label.trim().is_empty() || label.contains([')', '(', '\n']) || inner[..open].ends_with('\\') {
            return;
        }
        // The parentheses must be plain text, not the end of another node.
        if n.prev_named_sibling().is_some_and(|p| p.end_byte() > open) {
            return;
        }
        let mut f = finding(open, n.end_byte(), format!("`({label})[{target}]` is a link written backwards; write `[{label}]({target})`"));
        f.edits = vec![TextEdit::new(open, n.end_byte(), format!("[{label}]({target})"))];
        out.push(f);
    }
}

/// Byte ranges an inline root's bare text must avoid.
fn non_text_ranges(root: Node<'_>) -> Vec<(usize, usize)> {
    preorder(root)
        .into_iter()
        .filter(|n| {
            matches!(
                n.kind(),
                "inline_link" | "full_reference_link" | "collapsed_reference_link" | "shortcut_link" | "image" | "uri_autolink" | "email_autolink" | "code_span" | "html_tag" | "latex_block"
            )
        })
        .map(|n| (n.start_byte(), n.end_byte()))
        .collect()
}

pub struct NoBareUrls;

impl NativeRule for NoBareUrls {
    fn id(&self) -> &'static str {
        "md/no-bare-urls"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["inline"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let root = node.get_inner_node();
        if !inline_root(root) {
            return;
        }
        let src = ctx.source;
        let skip = non_text_ranges(root);
        let mut from = root.start_byte();
        let end = root.end_byte();
        while let Some(off) = ["https://", "http://"].iter().filter_map(|s| src[from..end].find(s)).min() {
            let start = from + off;
            let mut stop = src[start..end].find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '`' | '"')).map(|i| start + i).unwrap_or(end);
            // Trailing punctuation ends the sentence, not the URL; a `)` only
            // belongs to the URL when it closes a `(` inside it.
            loop {
                let url = &src[start..stop];
                match url.chars().last() {
                    Some('.' | ',' | ':' | ';' | '!' | '?' | '\'' | '*' | '_' | '~') => stop -= 1,
                    Some(')') if url.matches('(').count() < url.matches(')').count() => stop -= 1,
                    _ => break,
                }
            }
            from = stop.max(start + 1);
            if stop <= start + "http://".len() {
                continue;
            }
            if skip.iter().any(|(a, b)| start < *b && stop > *a) {
                continue;
            }
            // Inside an HTML attribute that spans lines, or already bracketed.
            let prev = src[..start].chars().last();
            // `git+https://…` is another scheme; `](https://a b)` is a link
            // the parser rejected, whose URL wrapping in `<>` would break.
            if prev.is_some_and(|c| c.is_alphanumeric() || matches!(c, '=' | '"' | '\'' | '<' | '/' | '\\' | '+' | '-' | '.')) || src[..start].ends_with("](") {
                continue;
            }
            let url = &src[start..stop];
            let mut f = finding(start, stop, format!("Bare URL `{url}`; write it as `<{url}>` so every renderer links it"));
            f.edits = vec![TextEdit::new(start, stop, format!("<{url}>"))];
            out.push(f);
        }
    }
}

pub struct DescriptiveLinkText;

fn normalise_words(t: &str) -> String {
    t.chars().map(|c| if c.is_alphanumeric() || c.is_whitespace() { c.to_lowercase().next().unwrap_or(c) } else { ' ' }).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
}

impl NativeRule for DescriptiveLinkText {
    fn id(&self) -> &'static str {
        "md/descriptive-link-text"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["inline_link", "full_reference_link", "collapsed_reference_link"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let Some(link_text) = named_children(n).into_iter().find(|c| c.kind() == "link_text") else { return };
        let prohibited: Vec<String> = match ctx.options.get("prohibited_texts").and_then(|v| v.as_array()) {
            Some(a) => a.iter().filter_map(|v| v.as_str()).map(normalise_words).collect(),
            None => ["click here", "here", "link", "more"].iter().map(|s| s.to_string()).collect(),
        };
        let mut plain = String::new();
        plain_text(link_text, ctx.source, &mut plain);
        let words = normalise_words(&plain);
        if prohibited.contains(&words) {
            out.push(finding(n.start_byte(), n.end_byte(), format!("\"{}\" doesn't say where the link goes; describe its target", plain.trim())));
        }
    }
}

pub struct NoEmphasisAsHeading;

impl NativeRule for NoEmphasisAsHeading {
    fn id(&self) -> &'static str {
        "md/no-emphasis-as-heading"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["paragraph"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let p = node.get_inner_node();
        let src = ctx.source;
        if !p.parent().is_some_and(|x| matches!(x.kind(), "section" | "document")) {
            return;
        }
        let Some(inline) = named_children(p).into_iter().find(|c| c.kind() == "inline") else { return };
        let raw = text(inline, src);
        if raw.contains('\n') || !raw.starts_with(['*', '_']) {
            return;
        }
        let tree = parse_inline(src, inline);
        let root = tree.root().get_inner_node();
        let kids = named_children(root);
        let [only] = kids.as_slice() else { return };
        if !matches!(only.kind(), "emphasis" | "strong_emphasis") || only.start_byte() != 0 || only.end_byte() != raw.trim_end().len() {
            return;
        }
        // Emphasis around a link is a call to action, not a section title.
        if preorder(*only).iter().any(|n| matches!(n.kind(), "inline_link" | "full_reference_link" | "collapsed_reference_link" | "shortcut_link" | "image" | "html_tag")) {
            return;
        }
        let mut plain = String::new();
        plain_text(*only, raw, &mut plain);
        let punctuation = ctx.options.get("punctuation").and_then(|v| v.as_str()).unwrap_or(".,;:!?。，；：！？");
        if plain.trim().chars().last().is_none_or(|c| punctuation.contains(c)) {
            return;
        }
        out.push(finding(inline.start_byte(), inline.end_byte(), format!("\"{}\" is emphasis standing in for a heading; use a heading", plain.trim())));
    }
}

// ------------------------------------------- links against the whole document

/// A reference label as Markdown matches it: case-folded, inner whitespace
/// collapsed.
fn label_key(t: &str) -> String {
    t.trim().trim_start_matches('[').trim_end_matches(']').split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// The inline trees of the document whose text contains one of `needles`
/// (parsing is the cost; most inline content has no link at all), with the
/// host offset each starts at.
fn inline_trees(doc: Node<'_>, src: &str, needles: &[&str]) -> Vec<(usize, codelaw_lang::Root)> {
    preorder(doc)
        .into_iter()
        .filter(|n| n.kind() == "inline" && needles.iter().any(|x| text(*n, src).contains(x)))
        .filter(|n| !n.parent().is_some_and(|p| definition_paragraph(p, src)))
        .map(|n| (n.start_byte(), parse_inline(src, n)))
        .collect()
}

/// `[label]: destination` on one line: the label and the destination.
fn definition_line(line: &str) -> Option<(String, String)> {
    let t = line.strip_prefix("   ").or_else(|| line.strip_prefix("  ")).or_else(|| line.strip_prefix(' ')).unwrap_or(line);
    let rest = t.strip_prefix('[')?;
    let close = rest.find("]:")?;
    let label = &rest[..close];
    if label.trim().is_empty() || label.contains(['[', ']']) {
        return None;
    }
    let dest = rest[close + 2..].split_whitespace().next()?;
    Some((label_key(label), dest.to_owned()))
}

/// A paragraph made only of `[label]: #fragment` lines. The block grammar
/// reads a definition whose destination starts with `#` as a paragraph;
/// Markdown makes it a definition.
fn definition_paragraph(p: Node<'_>, src: &str) -> bool {
    p.kind() == "paragraph" && text(p, src).lines().filter(|l| !l.trim().is_empty()).all(|l| definition_line(l).is_some())
}

/// One reference definition: its label, destination and byte range (the
/// line without its line break), and whether it is a block of its own.
struct Definition {
    label: String,
    dest: Option<String>,
    start: usize,
    end: usize,
    top_level: bool,
}

/// The label a reference link or image uses, and whether it is a shortcut.
fn reference_label(n: Node<'_>, src: &str) -> Option<(String, bool)> {
    let kids = named_children(n);
    let label = kids.iter().find(|c| c.kind() == "link_label");
    let text_node = kids.iter().find(|c| matches!(c.kind(), "link_text" | "image_description"));
    match n.kind() {
        "full_reference_link" => label.map(|l| (label_key(text(*l, src)), false)),
        "collapsed_reference_link" => text_node.map(|t| (label_key(text(*t, src)), false)),
        "shortcut_link" => text_node.map(|t| (label_key(text(*t, src)), true)),
        "image" if kids.iter().all(|c| c.kind() != "link_destination") => {
            let t = text(n, src);
            if let Some(l) = label {
                let l = label_key(text(*l, src));
                if l.is_empty() { text_node.map(|t| (label_key(text(*t, src)), false)) } else { Some((l, false)) }
            } else if t.ends_with("]") && !t.ends_with(")") {
                text_node.map(|x| (label_key(text(*x, src)), true))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn definitions(doc: Node<'_>, src: &str) -> Vec<Definition> {
    let mut out = Vec::new();
    for n in preorder(doc) {
        let top_level = n.parent().is_some_and(|p| matches!(p.kind(), "section" | "document"));
        if n.kind() == "link_reference_definition" {
            let kids = named_children(n);
            let Some(label) = kids.iter().find(|c| c.kind() == "link_label") else { continue };
            out.push(Definition {
                label: label_key(text(*label, src)),
                dest: kids.iter().find(|c| c.kind() == "link_destination").map(|d| text(*d, src).to_owned()),
                start: n.start_byte(),
                end: n.start_byte() + text(n, src).trim_end().len(),
                top_level,
            });
        } else if definition_paragraph(n, src) {
            let mut at = n.start_byte();
            for line in text(n, src).split_inclusive('\n') {
                if let Some((label, dest)) = definition_line(line) {
                    out.push(Definition { label, dest: Some(dest), start: at, end: at + line.trim_end().len(), top_level });
                }
                at += line.len();
            }
        }
    }
    out
}

fn string_list(ctx: &NativeContext<'_>, key: &str, default: &[&str]) -> Vec<String> {
    match ctx.options.get(key).and_then(|v| v.as_array()) {
        Some(a) => a.iter().filter_map(|v| v.as_str()).map(str::to_owned).collect(),
        None => default.iter().map(|s| s.to_string()).collect(),
    }
}

pub struct ReferenceLinksImages;

impl NativeRule for ReferenceLinksImages {
    fn id(&self) -> &'static str {
        "md/reference-links-images"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let doc = node.get_inner_node();
        let src = ctx.source;
        let defined: BTreeSet<String> = definitions(doc, src).into_iter().map(|d| d.label).collect();
        let ignored: BTreeSet<String> = string_list(ctx, "ignored_labels", &["x"]).iter().map(|s| label_key(s)).collect();
        let shortcuts = ctx.options.get("shortcut_syntax").and_then(|v| v.as_bool()).unwrap_or(false);
        let needles: &[&str] = if shortcuts { &["["] } else { &["]["] };
        for (base, tree) in inline_trees(doc, src, needles) {
            let isrc = tree.get_text();
            for n in preorder(tree.root().get_inner_node()) {
                let Some((label, shortcut)) = reference_label(n, isrc) else { continue };
                if (shortcut && !shortcuts) || label.is_empty() || defined.contains(&label) || ignored.contains(&label) {
                    continue;
                }
                out.push(finding(base + n.start_byte(), base + n.end_byte(), format!("No definition `[{label}]: …` in this document; the reference renders as plain text")));
            }
        }
    }
}

pub struct LinkImageReferenceDefinitions;

impl NativeRule for LinkImageReferenceDefinitions {
    fn id(&self) -> &'static str {
        "md/link-image-reference-definitions"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let doc = node.get_inner_node();
        let src = ctx.source;
        let ignored: BTreeSet<String> = string_list(ctx, "ignored_definitions", &["//"]).iter().map(|s| label_key(s)).collect();
        let mut used: BTreeSet<String> = BTreeSet::new();
        let defs = definitions(doc, src);
        if defs.is_empty() {
            return;
        }
        for (_, tree) in inline_trees(doc, src, &["]"]) {
            let isrc = tree.get_text();
            for n in preorder(tree.root().get_inner_node()) {
                if let Some((label, _)) = reference_label(n, isrc) {
                    used.insert(label);
                }
            }
        }
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for def in defs {
            let label = def.label;
            if ignored.contains(&label) {
                continue;
            }
            let first = seen.insert(label.clone());
            let message = if !first {
                format!("`[{label}]` is defined again; only the first definition counts")
            } else if !used.contains(&label) {
                format!("Nothing refers to `[{label}]`")
            } else {
                continue;
            };
            let mut f = finding(def.start, def.end, message);
            // Remove the whole line, when the definition is a block of its own.
            let own_line = def.start == 0 || src[..def.start].ends_with('\n');
            if own_line && def.top_level {
                let line_end = src[def.end..].find('\n').map(|i| def.end + i + 1).unwrap_or(src.len());
                if src[def.end..line_end].trim().is_empty() {
                    f.edits = vec![TextEdit::delete(def.start, line_end)];
                }
            }
            out.push(f);
        }
    }
}

/// GitHub's heading anchor: lower-cased, punctuation dropped, each space a
/// dash.
fn slug(t: &str) -> String {
    t.to_lowercase().chars().filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | ' ')).map(|c| if c == ' ' { '-' } else { c }).collect()
}

fn percent_decode(t: &str) -> String {
    let b = t.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        // Hex digits are checked byte by byte: slicing `t` after a `%` could
        // split a multi-byte character (`#%aé`) and panic.
        if b[i] == b'%'
            && i + 3 <= b.len()
            && let (Some(hi), Some(lo)) = (hex_digit(b[i + 1]), hex_digit(b[i + 2]))
        {
            out.push(hi << 4 | lo);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| t.to_owned())
}

fn hex_digit(b: u8) -> Option<u8> {
    (b as char).to_digit(16).map(|d| d as u8)
}

/// `id="x"` and `name="x"` attributes anywhere in the document.
fn html_anchors(src: &str, out: &mut BTreeSet<String>) {
    let lower = src.to_ascii_lowercase();
    for attr in ["id", "name"] {
        let mut from = 0;
        while let Some(off) = lower[from..].find(attr) {
            let at = from + off;
            from = at + attr.len();
            let before = lower[..at].chars().last();
            if !before.is_some_and(|c| c.is_whitespace()) {
                continue;
            }
            let rest = src[from..].trim_start();
            let Some(rest) = rest.strip_prefix('=') else { continue };
            let rest = rest.trim_start();
            let value: String = match rest.chars().next() {
                Some(q @ ('"' | '\'')) => rest[1..].chars().take_while(|c| *c != q).collect(),
                _ => rest.chars().take_while(|c| !c.is_whitespace() && *c != '>').collect(),
            };
            if !value.is_empty() {
                out.insert(value);
            }
        }
    }
}

/// `#L20`, `#L19C5-L21C11`: GitHub's line anchors.
fn line_anchor(f: &str) -> bool {
    let part = |p: &str| {
        let Some(r) = p.strip_prefix('L') else { return false };
        let (l, c) = r.split_once('C').unwrap_or((r, "1"));
        !l.is_empty() && !c.is_empty() && l.bytes().all(|b| b.is_ascii_digit()) && c.bytes().all(|b| b.is_ascii_digit())
    };
    match f.split_once('-') {
        Some((a, b)) => part(a) && part(b),
        None => part(f),
    }
}

pub struct LinkFragments;

impl NativeRule for LinkFragments {
    fn id(&self) -> &'static str {
        "md/link-fragments"
    }
    fn languages(&self) -> &'static [Language] {
        MD
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let doc = node.get_inner_node();
        let src = ctx.source;
        // Most documents link to no fragment; skip building the anchors.
        if !src.contains("(#") && !src.contains("(<#") && !src.contains("]:") {
            return;
        }
        let ignore_case = ctx.options.get("ignore_case").and_then(|v| v.as_bool()).unwrap_or(false);
        let mut anchors: BTreeSet<String> = BTreeSet::new();
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for h in headings(doc) {
            let mut t = heading_text(&h, src);
            // `# Title {#custom-id}`
            if let Some(open) = t.rfind("{#")
                && t.ends_with('}')
            {
                anchors.insert(t[open + 2..t.len() - 1].to_owned());
                t = t[..open].trim_end().to_owned();
            }
            let s = slug(&t);
            let n = counts.entry(s.clone()).or_insert(0);
            anchors.insert(if *n == 0 { s.clone() } else { format!("{s}-{n}") });
            *n += 1;
        }
        html_anchors(src, &mut anchors);
        anchors.insert("top".into());
        let anchors_folded: BTreeSet<String> = anchors.iter().map(|a| a.to_lowercase()).collect();
        let mut check = |dest: &str, start: usize, end: usize| {
            let Some(fragment) = dest.trim_start_matches('<').trim_end_matches('>').strip_prefix('#') else { return };
            if fragment.is_empty() || line_anchor(fragment) {
                return;
            }
            let f = percent_decode(fragment);
            let ok = anchors.contains(&f) || (ignore_case && anchors_folded.contains(&f.to_lowercase()));
            if !ok {
                let hint = if anchors_folded.contains(&f.to_lowercase()) { "; anchors are lower-case" } else { "" };
                out.push(finding(start, end, format!("No heading or anchor in this document has the id `{f}`{hint}")));
            }
        };
        for (base, tree) in inline_trees(doc, src, &["(#", "(<#"]) {
            let isrc = tree.get_text();
            for n in preorder(tree.root().get_inner_node()).into_iter().filter(|n| n.kind() == "inline_link") {
                if let Some(d) = named_children(n).into_iter().find(|c| c.kind() == "link_destination") {
                    check(text(d, isrc), base + n.start_byte(), base + n.end_byte());
                }
            }
        }
        for d in definitions(doc, src) {
            if let Some(dest) = &d.dest {
                check(dest, d.start, d.end);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run a rule the way the engine does on a Markdown file: the block tree,
    /// then each inline region's tree.
    fn found(rule: &dyn NativeRule, src: &str) -> Vec<String> {
        let mut out = crate::run(rule, Language::Markdown, src);
        let root = codelaw_lang::parse(Language::Markdown, src);
        for r in codelaw_lang::regions(Language::Markdown, &root, src, Default::default()) {
            if r.grammar != Grammar::MarkdownInline {
                continue;
            }
            let tree = codelaw_lang::parse_grammar(Grammar::MarkdownInline, &r.text);
            let options = BTreeMap::new();
            let data = BTreeMap::new();
            let ctx = NativeContext { source: &r.text, language: Language::Markdown, semantic: None, options: &options, data: &data, file: &crate::DetachedFile::default() };
            for n in tree.root().dfs() {
                if rule.kinds().contains(&n.kind().as_ref()) {
                    let mut o = Vec::new();
                    rule.check(n, &ctx, &mut o);
                    out.extend(o.into_iter().map(|mut f| {
                        f.start += r.host_start;
                        f.end += r.host_start;
                        f
                    }));
                }
            }
        }
        out.into_iter().map(|f| src[f.start..f.end].to_owned()).collect()
    }

    #[test]
    fn heading_rules() {
        assert_eq!(found(&HeadingIncrement, "# A\n\n### B\n\n#### C\n\n## D\n"), ["### B"]);
        assert_eq!(found(&SingleH1, "# A\n\n# B\n"), ["# B"]);
        assert_eq!(found(&SingleH1, "---\ntitle: x\n---\n# A\n"), ["# A"]);
        assert_eq!(found(&NoDuplicateHeading, "# A\n## Added\n# B\n## Added\n## Added\n"), ["## Added"]);
        assert!(found(&FirstLineHeading, "<!-- c -->\n# A\n").is_empty());
        assert_eq!(found(&FirstLineHeading, "Text\n\n# A\n"), ["Text"]);
    }

    #[test]
    fn code_and_tables() {
        assert_eq!(found(&CommandsShowOutput, "```sh\n$ ls\n$ pwd\n```\n\n```sh\n$ ls\nfile\n```\n").len(), 1);
        assert_eq!(found(&TableColumnCount, "| a | b |\n|---|---|\n| 1 | 2 | 3 |\n| 4 | 5 |\n"), ["| 1 | 2 | 3 |"]);
    }

    #[test]
    fn inline_rules() {
        assert_eq!(found(&NoReversedLinks, "See (the docs)[https://x.y] and (a)[^1].\n"), ["(the docs)[https://x.y]"]);
        assert_eq!(
            found(&NoBareUrls, "Go to https://a.b/c. Not <https://d.e> or [x](https://f.g) or `https://h.i` or (https://j.k/l_(m)).\n"),
            ["https://a.b/c", "https://j.k/l_(m)"]
        );
        assert_eq!(found(&DescriptiveLinkText, "Read [here](x) or [the guide](y) or [Click  here!](z).\n"), ["[here](x)", "[Click  here!](z)"]);
        assert_eq!(found(&NoEmphasisAsHeading, "**Usage**\n\ntext **b**\n\n*Note.*\n"), ["**Usage**"]);
    }

    #[test]
    fn references_and_fragments() {
        let src = "# My Title\n\n[a][one] [b][two] [c] ![d][img] [e][]\n\n[one]: x\n[img]: y\n[unused]: z\n[one]: w\n[//]: # (comment)\n";
        assert_eq!(found(&ReferenceLinksImages, src), ["[b][two]", "[e][]"]);
        assert_eq!(found(&LinkImageReferenceDefinitions, src), ["[unused]: z", "[one]: w"]);
        let src = "# My Title\n\n## Usage: `x`\n\n## Usage: `x`\n\n<a id=\"custom\"></a>\n\n[1](#my-title) [2](#usage-x-1) [3](#nope) [4](#My-Title) [5](#custom) [6](#L10) [7](#top)\n";
        assert_eq!(found(&LinkFragments, src), ["[3](#nope)", "[4](#My-Title)"]);
    }
}
