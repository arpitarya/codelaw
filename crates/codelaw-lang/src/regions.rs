//! Embedded regions ("injections", SR-LINTER decision 3): code of one
//! language inside a host file of another, linted with its own rules and
//! mapped back to host positions.

use crate::{Grammar, Root, SyntaxNode};
use codelaw_core::Language;

/// One embedded region of a host file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Region {
    pub language: Language,
    /// The grammar it parses with: the language's, or Markdown's inline
    /// grammar for an `inline` node.
    pub grammar: Grammar,
    /// The text that is parsed. It may start with a synthetic prefix that is
    /// not in the host (a `style=""` attribute is parsed as `a{…}`).
    pub text: String,
    /// Bytes of synthetic prefix at the start of `text`.
    pub prefix_len: usize,
    /// Host byte offset of the first non-synthetic byte of `text`.
    pub host_start: usize,
    /// Host byte offset one past the region's last byte.
    pub host_end: usize,
    /// Host code standing inside the region (a template's `${…}`), each
    /// replaced in `text` by a placeholder, in order.
    pub holes: Vec<Hole>,
}

/// A stretch of host code inside a region, which `text` holds as a
/// placeholder: nothing in it maps to the host, and no fix may touch it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hole {
    /// Offset of the placeholder in `text`.
    pub at: usize,
    /// Bytes of placeholder in `text`.
    pub len: usize,
    /// Bytes of host code it stands for.
    pub host_len: usize,
}

impl Region {
    fn plain(language: Language, host: &str, start: usize, end: usize) -> Region {
        Region { language, grammar: Grammar::for_language(language), text: host[start..end].to_owned(), prefix_len: 0, host_start: start, host_end: end, holes: Vec::new() }
    }

    /// Map a region byte offset to the host. Offsets in the synthetic prefix
    /// or suffix, or inside a hole's placeholder, have no host position; a
    /// hole's two edges map to the edges of its host code.
    pub fn to_host(&self, offset: usize) -> Option<usize> {
        let inner = offset.checked_sub(self.prefix_len)?;
        let mut host = self.host_start + inner;
        for h in &self.holes {
            if offset <= h.at {
                break;
            }
            if offset < h.at + h.len {
                return None;
            }
            host = host + h.host_len - h.len;
        }
        (host <= self.host_end).then_some(host)
    }

    /// Map a finding's range to the host. A finding inside a placeholder is
    /// about the placeholder, not the host's code, so it has none.
    pub fn span_to_host(&self, start: usize, end: usize) -> Option<(usize, usize)> {
        let within = |h: &Hole| start >= h.at && end <= h.at + h.len && !(start == end && (start == h.at || start == h.at + h.len));
        if self.holes.iter().any(within) {
            return None;
        }
        Some((self.to_host(start)?, self.to_host(end)?))
    }

    /// Map an edit's range to the host: none when it would change a hole's
    /// host code.
    pub fn edit_to_host(&self, start: usize, end: usize) -> Option<(usize, usize)> {
        let touches = |h: &Hole| (start < h.at + h.len && end > h.at) || (start == end && start > h.at && start < h.at + h.len);
        if self.holes.iter().any(touches) {
            return None;
        }
        Some((self.to_host(start)?, self.to_host(end)?))
    }
}

/// What the caller wants extracted.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegionOptions {
    /// `markdown.lint_code_blocks` (L3 decision 8); off by default.
    pub lint_code_blocks: bool,
    /// `[lint] tagged_templates`: `` css`…` `` and `` html`…` `` in
    /// JavaScript and TypeScript; off by default.
    pub tagged_templates: bool,
}

/// The embedded regions of a host file, in host order.
pub fn regions(host_language: Language, root: &Root, source: &str, options: RegionOptions) -> Vec<Region> {
    let mut out = Vec::new();
    match host_language {
        Language::Html => html_regions(root.root(), source, &mut out),
        Language::Markdown => markdown_regions(root.root(), source, options, &mut out),
        Language::Javascript | Language::Jsx | Language::Typescript | Language::Tsx if options.tagged_templates => {
            template_regions(root.root(), source, &mut out)
        }
        _ => {}
    }
    out.sort_by_key(|r| r.host_start);
    out
}

fn attributes<'r>(start_tag: &SyntaxNode<'r>) -> Vec<(String, Option<SyntaxNode<'r>>)> {
    start_tag
        .children()
        .filter(|c| c.kind() == "attribute")
        .map(|attr| {
            let name = attr
                .children()
                .find(|c| c.kind() == "attribute_name")
                .map(|n| n.text().to_ascii_lowercase())
                .unwrap_or_default();
            let value = attr.children().find_map(|c| match c.kind().as_ref() {
                "attribute_value" => Some(c),
                "quoted_attribute_value" => c.children().find(|v| v.kind() == "attribute_value"),
                _ => None,
            });
            (name, value)
        })
        .collect()
}

fn script_language(start_tag: &SyntaxNode<'_>) -> Option<Language> {
    let attrs = attributes(start_tag);
    let get = |key: &str| {
        attrs
            .iter()
            .find(|(n, _)| n == key)
            .map(|(_, v)| v.as_ref().map(|v| v.text().trim().to_ascii_lowercase()).unwrap_or_default())
    };
    if let Some(lang) = get("lang") {
        return match lang.as_str() {
            "ts" | "typescript" => Some(Language::Typescript),
            "tsx" => Some(Language::Tsx),
            "js" | "javascript" | "" => Some(Language::Javascript),
            "jsx" => Some(Language::Jsx),
            _ => None,
        };
    }
    match get("type").as_deref() {
        None | Some("") | Some("module") | Some("text/javascript") | Some("application/javascript") => {
            Some(Language::Javascript)
        }
        Some("application/json") | Some("importmap") | Some("application/ld+json") => Some(Language::Json),
        _ => None,
    }
}

fn html_regions(root: SyntaxNode<'_>, source: &str, out: &mut Vec<Region>) {
    for node in root.dfs() {
        match node.kind().as_ref() {
            "script_element" | "style_element" => {
                let Some(start_tag) = node.children().find(|c| c.kind() == "start_tag") else { continue };
                let language = if node.kind() == "style_element" {
                    Some(Language::Css)
                } else {
                    script_language(&start_tag)
                };
                let (Some(language), Some(raw)) = (language, node.children().find(|c| c.kind() == "raw_text"))
                else {
                    continue;
                };
                let r = raw.range();
                out.push(Region::plain(language, source, r.start, r.end));
            }
            "start_tag" | "self_closing_tag" => {
                for (name, value) in attributes(&node) {
                    if name != "style" {
                        continue;
                    }
                    let Some(value) = value else { continue };
                    let r = value.range();
                    out.push(Region {
                        language: Language::Css,
                        grammar: Grammar::Css,
                        text: format!("a{{{}}}", &source[r.clone()]),
                        prefix_len: 2,
                        host_start: r.start,
                        host_end: r.end,
                        holes: Vec::new(),
                    });
                }
            }
            _ => {}
        }
    }
}

fn markdown_regions(root: SyntaxNode<'_>, source: &str, options: RegionOptions, out: &mut Vec<Region>) {
    for node in root.dfs() {
        match node.kind().as_ref() {
            "inline" => {
                let r = node.range();
                out.push(Region { grammar: Grammar::MarkdownInline, ..Region::plain(Language::Markdown, source, r.start, r.end) });
            }
            "minus_metadata" => {
                let r = node.range();
                let text = &source[r.clone()];
                let Some(first_nl) = text.find('\n') else { continue };
                let body_start = r.start + first_nl + 1;
                let mut body_end = r.end;
                let body = &source[body_start..body_end];
                let trimmed = body.trim_end_matches(['\n', '\r']);
                if let Some(last_nl) = trimmed.rfind('\n').map(|i| i + 1).or(Some(0))
                    && trimmed[last_nl..].trim() == "---"
                {
                    body_end = body_start + last_nl;
                }
                out.push(Region::plain(Language::Yaml, source, body_start, body_end));
            }
            "fenced_code_block" if options.lint_code_blocks => {
                // Only fences at the top level: inside a list or quote the
                // content carries continuation markers that are not code.
                let top_level = node.parent().is_some_and(|p| matches!(p.kind().as_ref(), "section" | "document"));
                if !top_level {
                    continue;
                }
                let tag = node
                    .children()
                    .find(|c| c.kind() == "info_string")
                    .and_then(|i| i.children().find(|c| c.kind() == "language"))
                    .map(|l| l.text().to_string());
                let Some(language) = tag.as_deref().and_then(Language::from_fence_tag) else { continue };
                let Some(content) = node.children().find(|c| c.kind() == "code_fence_content") else { continue };
                let r = content.range();
                out.push(Region::plain(language, source, r.start, r.end));
            }
            _ => {}
        }
    }
}

/// `` css`…` `` and `` html`…` `` tagged templates (SR-LINTER decision 3).
/// Each `${…}` becomes a hole whose placeholder keeps the region parseable.
fn template_regions(root: SyntaxNode<'_>, source: &str, out: &mut Vec<Region>) {
    for node in root.dfs() {
        if node.kind() != "call_expression" {
            continue;
        }
        let (Some(tag), Some(tpl)) = (node.field("function"), node.field("arguments")) else { continue };
        if tpl.kind() != "template_string" || tag.kind() != "identifier" {
            continue;
        }
        let language = match tag.text().as_ref() {
            "css" => Language::Css,
            "html" => Language::Html,
            _ => continue,
        };
        let r = tpl.range();
        if r.end < r.start + 2 {
            continue;
        }
        let (start, end) = (r.start + 1, r.end - 1);
        let subs: Vec<(usize, usize)> = tpl.children().filter(|c| c.kind() == "template_substitution").map(|c| (c.range().start, c.range().end)).collect();
        let mut text = String::with_capacity(end - start);
        let mut holes = Vec::new();
        let mut at = start;
        for (n, &(a, b)) in subs.iter().enumerate() {
            text.push_str(&source[at..a]);
            let placeholder = match language {
                Language::Css => css_placeholder(source, start, end, a, b, n),
                _ => "${}".to_owned(),
            };
            holes.push(Hole { at: text.len(), len: placeholder.len(), host_len: b - a });
            text.push_str(&placeholder);
            at = b;
        }
        text.push_str(&source[at..end]);
        out.push(Region { language, grammar: Grammar::for_language(language), text, prefix_len: 0, host_start: start, host_end: end, holes });
    }
}

fn ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b >= 0x80
}

/// What stands for the `${…}` at `a..b` of a CSS template (`start..end`),
/// chosen by where it is, so the stylesheet still parses and no rule
/// reports the placeholder itself:
///
/// | where | placeholder |
/// |---|---|
/// | a statement of its own, at the top level | a comment |
/// | a statement of its own, in a block | a custom property declaration |
/// | a property name | a custom property name |
/// | a selector | a class (an identifier when glued to one) |
/// | an at-rule prelude | `all` for `@media`, otherwise an identifier |
/// | a value glued to a unit, or after `#` | a number |
/// | any other value | `var(--…)` |
fn css_placeholder(src: &str, start: usize, end: usize, a: usize, b: usize, n: usize) -> String {
    let bytes = src.as_bytes();
    let name = format!("codelaw-tpl-{n}");
    let prev = src[start..a].trim_end().bytes().last();
    let after = src[b..end].trim_start();
    let next = after.bytes().next();
    let glued_before = a > start && (ident_byte(bytes[a - 1]) || bytes[a - 1] == b'#');
    let glued_after = b < end && (ident_byte(bytes[b]) || bytes[b] == b'%');
    // Brace depth, and where the statement holding the hole starts.
    let mut depth = 0i32;
    let mut stmt = start;
    for (i, c) in src[start..a].bytes().enumerate() {
        match c {
            b'{' => {
                depth += 1;
                stmt = start + i + 1;
            }
            b'}' => {
                depth -= 1;
                stmt = start + i + 1;
            }
            b';' => stmt = start + i + 1,
            _ => {}
        }
    }
    let head = src[stmt..a].trim();
    // What ends the statement: `{` opens a block (a selector or a prelude);
    // `;` or `}` ends a declaration.
    let opens_block = after.bytes().find(|c| matches!(c, b'{' | b';' | b'}')) == Some(b'{');
    if head.is_empty() && !glued_after && matches!(prev, None | Some(b'{' | b'}' | b';')) && !opens_block {
        if next == Some(b':') {
            return format!("--{name}");
        }
        if depth <= 0 {
            return format!("/* {name} */");
        }
        return if matches!(next, Some(b';' | b'}') | None) { format!("--{name}: 0") } else { format!("--{name}: 0;") };
    }
    if let Some(rest) = head.strip_prefix('@') {
        let at = rest.split(|c: char| !c.is_ascii_alphanumeric() && c != '-').next().unwrap_or("").to_ascii_lowercase();
        return if at == "media" && !glued_before && !glued_after { "all".into() } else { format!("codelaw-tpl{n}") };
    }
    if opens_block && !head.contains(':') || opens_block && head.is_empty() || opens_block && !head.contains(';') && next != Some(b';') && depth <= 0 {
        return if glued_before || glued_after { format!("codelaw-tpl{n}") } else { format!(".{name}") };
    }
    if a > start && bytes[a - 1] == b'#' {
        return "000".into();
    }
    if glued_before {
        return format!("tpl{n}");
    }
    if glued_after {
        return "0".into();
    }
    format!("var(--{name})")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn texts(lang: Language, src: &str, options: RegionOptions) -> Vec<(Language, String)> {
        let root = parse(lang, src);
        regions(lang, &root, src, options).into_iter().filter(|r| r.grammar != Grammar::MarkdownInline).map(|r| (r.language, r.text)).collect()
    }

    #[test]
    fn markdown_inline_content_is_a_region() {
        let src = "# A [link](./x.md)

See *this* and `code`.
";
        let root = parse(Language::Markdown, src);
        let inline: Vec<String> =
            regions(Language::Markdown, &root, src, RegionOptions::default()).into_iter().filter(|r| r.grammar == Grammar::MarkdownInline).map(|r| r.text).collect();
        assert_eq!(inline, ["A [link](./x.md)", "See *this* and `code`."]);
        let tree = crate::parse_grammar(Grammar::MarkdownInline, &inline[0]);
        assert!(tree.root().get_inner_node().to_sexp().contains("inline_link"), "{}", tree.root().get_inner_node().to_sexp());
    }

    #[test]
    fn html_scripts_styles_and_style_attributes() {
        let src = "<script type=\"module\">let a = 1;</script>\n<script lang=\"ts\">let b: number = 2;</script>\n<script type=\"application/json\">{\"a\":1}</script>\n<script type=\"text/template\">x</script>\n<style>a { color: red }</style>\n<p style=\"color: blue\">x</p>\n";
        assert_eq!(
            texts(Language::Html, src, RegionOptions::default()),
            vec![
                (Language::Javascript, "let a = 1;".into()),
                (Language::Typescript, "let b: number = 2;".into()),
                (Language::Json, "{\"a\":1}".into()),
                (Language::Css, "a { color: red }".into()),
                (Language::Css, "a{color: blue}".into()),
            ]
        );
    }

    #[test]
    fn style_attribute_maps_back_to_host() {
        let src = "<p style=\"color: blue\">x</p>";
        let root = parse(Language::Html, src);
        let r = &regions(Language::Html, &root, src, RegionOptions::default())[0];
        let in_region = r.text.find("blue").unwrap();
        assert_eq!(r.to_host(in_region), Some(src.find("blue").unwrap()));
        assert_eq!(r.to_host(0), None);
    }

    fn templates(src: &str) -> Vec<Region> {
        let root = parse(Language::Typescript, src);
        regions(Language::Typescript, &root, src, RegionOptions { tagged_templates: true, ..Default::default() })
    }

    #[test]
    fn tagged_templates_only_when_asked() {
        let src = "const s = css`a { color: red }`;\n";
        let root = parse(Language::Javascript, src);
        assert!(regions(Language::Javascript, &root, src, RegionOptions::default()).is_empty());
        let r = &templates(src)[0];
        assert_eq!((r.language, r.text.as_str()), (Language::Css, "a { color: red }"));
        assert_eq!(r.to_host(r.text.find("red").unwrap()), Some(src.find("red").unwrap()));
    }

    #[test]
    fn template_holes_map_around_the_host_code() {
        let src = "const s = css`a { color: ${c}; top: 1px }`;\nconst t = html`<p class=${k}>${x}</p>`; other`a{}`;\n";
        let rs = templates(src);
        assert_eq!(rs.len(), 2);
        let (c, h) = (&rs[0], &rs[1]);
        assert_eq!(c.text, "a { color: var(--codelaw-tpl-0); top: 1px }");
        assert_eq!(h.language, Language::Html);
        assert_eq!(h.text, "<p class=${}>${}</p>");
        // After the hole, offsets move by the difference in length.
        assert_eq!(c.to_host(c.text.find("top").unwrap()), Some(src.find("top").unwrap()));
        // Inside a placeholder: no host position, no finding, no edit.
        assert_eq!(c.to_host(c.text.find("codelaw").unwrap()), None);
        let (ha, hb) = (c.holes[0].at, c.holes[0].at + c.holes[0].len);
        assert_eq!(c.span_to_host(ha, hb), None);
        assert_eq!(c.edit_to_host(ha - 1, hb), None);
        // A finding around it maps; its edges are the `${` and the `}`.
        let decl = c.text.find("color").unwrap();
        assert_eq!(c.span_to_host(decl, hb), Some((src.find("color").unwrap(), src.find("};").unwrap() + 1)));
        assert!(c.edit_to_host(hb, hb).is_some(), "an insertion right after the hole is allowed");
    }

    #[test]
    fn css_placeholders_fit_where_they_stand() {
        let one = |body: &str| templates(&format!("css`{body}`"))[0].text.clone();
        assert_eq!(one("${base}"), "/* codelaw-tpl-0 */");
        assert_eq!(one("${base} a { top: 0 }"), ".codelaw-tpl-0 a { top: 0 }");
        assert_eq!(one("a { ${mixin} top: 0 }"), "a { --codelaw-tpl-0: 0; top: 0 }");
        assert_eq!(one("a { ${mixin}; }"), "a { --codelaw-tpl-0: 0; }");
        assert_eq!(one("a { ${p}: red }"), "a { --codelaw-tpl-0: red }");
        assert_eq!(one("${sel} { top: 0 }"), ".codelaw-tpl-0 { top: 0 }");
        assert_eq!(one("a${sel}:hover { top: 0 }"), "acodelaw-tpl0:hover { top: 0 }");
        assert_eq!(one("@media ${q} { a { top: 0 } }"), "@media all { a { top: 0 } }");
        assert_eq!(one("a { width: ${w}px; color: #${hex} }"), "a { width: 0px; color: #000 }");
    }

    #[test]
    fn nested_templates_are_regions_of_their_own() {
        let src = "html`<ul>${xs.map((x) => html`<li>${x}</li>`)}</ul>`;";
        let texts: Vec<String> = templates(src).into_iter().map(|r| r.text).collect();
        assert_eq!(texts, ["<ul>${}</ul>", "<li>${}</li>"]);
    }

    #[test]
    fn markdown_frontmatter_and_fences() {
        let src = "---\ntitle: x\n---\n# T\n\n```ts\nlet a = 1;\n```\n\n```text\nno\n```\n";
        assert_eq!(texts(Language::Markdown, src, RegionOptions::default()), vec![(Language::Yaml, "title: x\n".into())]);
        assert_eq!(
            texts(Language::Markdown, src, RegionOptions { lint_code_blocks: true, ..Default::default() }),
            vec![(Language::Yaml, "title: x\n".into()), (Language::Typescript, "let a = 1;\n".into())]
        );
    }
}
