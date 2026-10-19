//! `js/no-useless-escape` for strings and template literals: a backslash
//! before a character that means the same without it. Escapes inside
//! regular expressions are not checked here.

use crate::util::text;
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;

pub struct NoUselessEscape;

/// Characters an escape changes the meaning of, in any string.
const MEANINGFUL: &str = "\\nrvtbf0123456789xu\n\r\u{2028}\u{2029}";

impl NativeRule for NoUselessEscape {
    fn id(&self) -> &'static str {
        "js/no-useless-escape"
    }
    fn languages(&self) -> &'static [Language] {
        &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["escape_sequence"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let Some(parent) = n.parent() else { return };
        let delimiter = match parent.kind() {
            "string" => text(parent, ctx.source).chars().next().unwrap_or('"'),
            "template_string" => '`',
            _ => return,
        };
        let t = text(n, ctx.source);
        let Some(c) = t.strip_prefix('\\').and_then(|r| r.chars().next()) else { return };
        let needed = MEANINGFUL.contains(c) || c == delimiter || (delimiter == '`' && (c == '$' || c == '{'));
        if !needed {
            out.push(NativeFinding {
                start: n.start_byte(),
                end: n.end_byte(),
                message: format!("`\\{c}` is the same as `{c}`; the backslash does nothing"),
                edits: vec![TextEdit::delete(n.start_byte(), n.start_byte() + 1)],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_and_templates() {
        let src = r#"a = "\d \" \' \n"; b = '\" \''; c = `\$ \{ \# \``;"#;
        let found: Vec<String> = crate::run(&NoUselessEscape, Language::Javascript, src).into_iter().map(|f| src[f.start..f.end].to_owned()).collect();
        assert_eq!(found, [r"\d", r"\'", r#"\""#, r"\#"]);
    }
}
