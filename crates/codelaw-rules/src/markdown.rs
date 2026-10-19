//! A scan of a rule file's Markdown body into the pieces the template
//! names: headings, fenced code blocks, paragraphs and list items, each
//! with the byte offset it starts at.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    Heading { level: u8, text: String },
    Fence { info: String, code: String },
    /// A paragraph outside any list or quote.
    Paragraph { text: String, starts_strong: Option<String> },
    /// A top-level item of an ordered list.
    OrderedItem { text: String },
}

#[derive(Debug, Clone)]
pub struct Located {
    pub offset: usize,
    pub piece: Piece,
}

fn level(l: HeadingLevel) -> u8 {
    match l {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

pub fn scan(body: &str) -> Vec<Located> {
    let mut out = Vec::new();
    // What is being collected, and from where.
    let mut text = String::new();
    let mut start = 0;
    let mut heading: Option<u8> = None;
    let mut fence: Option<String> = None;
    let mut paragraph = false;
    let mut strong_first: Option<String> = None;
    let mut strong_open = false;
    let mut paragraph_has_text = false;
    let mut list_stack: Vec<bool> = Vec::new();
    let mut item_depth_text: Option<String> = None;
    let mut quote = 0;

    for (event, range) in Parser::new_ext(body, Options::empty()).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level: l, .. }) => {
                heading = Some(level(l));
                text.clear();
                start = range.start;
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = heading.take() {
                    out.push(Located { offset: start, piece: Piece::Heading { level, text: text.trim().to_owned() } });
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let info = match kind {
                    CodeBlockKind::Fenced(info) => info.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                fence = Some(info);
                text.clear();
                start = range.start;
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(info) = fence.take()
                    && list_stack.is_empty()
                    && quote == 0
                {
                    out.push(Located { offset: start, piece: Piece::Fence { info: info.trim().to_owned(), code: std::mem::take(&mut text) } });
                }
            }
            Event::Start(Tag::Paragraph) if list_stack.is_empty() && quote == 0 => {
                paragraph = true;
                paragraph_has_text = false;
                strong_first = None;
                text.clear();
                start = range.start;
            }
            Event::End(TagEnd::Paragraph) if paragraph => {
                paragraph = false;
                out.push(Located {
                    offset: start,
                    piece: Piece::Paragraph { text: text.trim().to_owned(), starts_strong: strong_first.take() },
                });
            }
            Event::Start(Tag::Strong) => {
                if paragraph && !paragraph_has_text {
                    strong_open = true;
                    strong_first = Some(String::new());
                }
            }
            Event::End(TagEnd::Strong) => strong_open = false,
            Event::Start(Tag::BlockQuote(_)) => quote += 1,
            Event::End(TagEnd::BlockQuote(_)) => quote -= 1,
            Event::Start(Tag::List(first)) => list_stack.push(first.is_some()),
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
            }
            Event::Start(Tag::Item) => {
                if list_stack.len() == 1 && list_stack[0] && quote == 0 {
                    item_depth_text = Some(String::new());
                    start = range.start;
                }
            }
            Event::End(TagEnd::Item) => {
                if list_stack.len() == 1
                    && let Some(t) = item_depth_text.take()
                {
                    out.push(Located { offset: start, piece: Piece::OrderedItem { text: t.split_whitespace().collect::<Vec<_>>().join(" ") } });
                }
            }
            Event::Text(t) | Event::Code(t) => {
                let piece: &str = &t;
                if let Some(item) = item_depth_text.as_mut() {
                    item.push_str(piece);
                }
                if paragraph {
                    if strong_open && let Some(s) = strong_first.as_mut() {
                        s.push_str(&t);
                    } else {
                        paragraph_has_text = true;
                    }
                }
                text.push_str(piece);
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(item) = item_depth_text.as_mut() {
                    item.push(' ');
                }
                text.push(if fence.is_some() { '\n' } else { ' ' });
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pieces_in_order() {
        let body = "Summary here.\n\n**Why:** because.\n\n```codelaw\nmatch: x\n```\n\n## How to fix\n\n1. First `step`.\n2. Second\n   step.\n\n## Examples\n\n### TypeScript\n\n#### Don't\n\n```ts\nlet a;\n```\n";
        let pieces: Vec<Piece> = scan(body).into_iter().map(|l| l.piece).collect();
        assert_eq!(
            pieces,
            vec![
                Piece::Paragraph { text: "Summary here.".into(), starts_strong: None },
                Piece::Paragraph { text: "Why: because.".into(), starts_strong: Some("Why:".into()) },
                Piece::Fence { info: "codelaw".into(), code: "match: x\n".into() },
                Piece::Heading { level: 2, text: "How to fix".into() },
                Piece::OrderedItem { text: "First step.".into() },
                Piece::OrderedItem { text: "Second step.".into() },
                Piece::Heading { level: 2, text: "Examples".into() },
                Piece::Heading { level: 3, text: "TypeScript".into() },
                Piece::Heading { level: 4, text: "Don't".into() },
                Piece::Fence { info: "ts".into(), code: "let a;\n".into() },
            ]
        );
    }

    #[test]
    fn longer_fences_nest() {
        let body = "````markdown\n```ts\nx\n```\n````\n";
        assert_eq!(scan(body)[0].piece, Piece::Fence { info: "markdown".into(), code: "```ts\nx\n```\n".into() });
    }
}
