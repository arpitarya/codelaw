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
}

impl Region {
    fn plain(language: Language, host: &str, start: usize, end: usize) -> Region {
        Region { language, grammar: Grammar::for_language(language), text: host[start..end].to_owned(), prefix_len: 0, host_start: start, host_end: end }
    }

    /// Map a region byte offset to the host. Offsets in the synthetic prefix
    /// or suffix have no host position.
    pub fn to_host(&self, offset: usize) -> Option<usize> {
        let inner = offset.checked_sub(self.prefix_len)?;
        let host = self.host_start + inner;
        (host <= self.host_end).then_some(host)
    }
}

/// What the caller wants extracted.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegionOptions {
    /// `markdown.lint_code_blocks` (L3 decision 8); off by default.
    pub lint_code_blocks: bool,
}

/// The embedded regions of a host file, in host order.
pub fn regions(host_language: Language, root: &Root, source: &str, options: RegionOptions) -> Vec<Region> {
    let mut out = Vec::new();
    match host_language {
        Language::Html => html_regions(root.root(), source, &mut out),
        Language::Markdown => markdown_regions(root.root(), source, options, &mut out),
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

    #[test]
    fn markdown_frontmatter_and_fences() {
        let src = "---\ntitle: x\n---\n# T\n\n```ts\nlet a = 1;\n```\n\n```text\nno\n```\n";
        assert_eq!(texts(Language::Markdown, src, RegionOptions::default()), vec![(Language::Yaml, "title: x\n".into())]);
        assert_eq!(
            texts(Language::Markdown, src, RegionOptions { lint_code_blocks: true }),
            vec![(Language::Yaml, "title: x\n".into()), (Language::Typescript, "let a = 1;\n".into())]
        );
    }
}
