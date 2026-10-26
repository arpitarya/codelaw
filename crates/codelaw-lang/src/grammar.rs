use ast_grep_core::Language as AgLanguage;
use ast_grep_core::matcher::{Pattern, PatternBuilder, PatternError};
use ast_grep_core::tree_sitter::{LanguageExt, StrDoc, TSLanguage};
use ast_grep_language as agl;
use codelaw_core::Language;
use std::borrow::Cow;

/// One tree-sitter grammar. JSX parses with the JavaScript grammar; TSX has
/// its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Grammar {
    JavaScript,
    TypeScript,
    Tsx,
    Python,
    Json,
    Yaml,
    Toml,
    Markdown,
    /// Markdown's inline grammar (links, emphasis, code spans), run on the
    /// block grammar's `inline` nodes.
    MarkdownInline,
    Css,
    Scss,
    Html,
}

impl Grammar {
    pub fn for_language(language: Language) -> Grammar {
        match language {
            Language::Javascript | Language::Jsx => Grammar::JavaScript,
            Language::Typescript => Grammar::TypeScript,
            Language::Tsx => Grammar::Tsx,
            Language::Python => Grammar::Python,
            Language::Json => Grammar::Json,
            Language::Yaml => Grammar::Yaml,
            Language::Toml => Grammar::Toml,
            Language::Markdown => Grammar::Markdown,
            Language::Css => Grammar::Css,
            Language::Scss => Grammar::Scss,
            Language::Html => Grammar::Html,
        }
    }

    /// Every grammar a language's rules compile for: the primary one, then
    /// any that parse its embedded parts.
    pub fn all_for_language(language: Language) -> &'static [Grammar] {
        match language {
            Language::Markdown => &[Grammar::Markdown, Grammar::MarkdownInline],
            Language::Javascript | Language::Jsx => &[Grammar::JavaScript],
            Language::Typescript => &[Grammar::TypeScript],
            Language::Tsx => &[Grammar::Tsx],
            Language::Python => &[Grammar::Python],
            Language::Json => &[Grammar::Json],
            Language::Yaml => &[Grammar::Yaml],
            Language::Toml => &[Grammar::Toml],
            Language::Css => &[Grammar::Css],
            Language::Scss => &[Grammar::Scss],
            Language::Html => &[Grammar::Html],
        }
    }
}

/// Delegates one method to the ast-grep language for each built-in grammar.
macro_rules! delegate {
    ($self:ident, $g:ident => $e:expr, toml => $t:expr) => {
        match $self {
            Grammar::JavaScript => { let $g = agl::JavaScript; $e }
            Grammar::TypeScript => { let $g = agl::TypeScript; $e }
            Grammar::Tsx => { let $g = agl::Tsx; $e }
            Grammar::Python => { let $g = agl::Python; $e }
            Grammar::Json => { let $g = agl::Json; $e }
            Grammar::Yaml => { let $g = agl::Yaml; $e }
            Grammar::Markdown | Grammar::MarkdownInline => { let $g = agl::Markdown; $e }
            Grammar::Css => { let $g = agl::Css; $e }
            Grammar::Html => { let $g = agl::Html; $e }
            Grammar::Toml => $t,
            // SCSS is codelaw's own grammar, patterned like CSS: it borrows
            // CSS's pattern syntax (`$` is not an SCSS identifier start in
            // patterns, so the CSS expando applies).
            Grammar::Scss => { let $g = agl::Css; $e }
        }
    };
}

fn toml_language() -> TSLanguage {
    tree_sitter_toml_ng::LANGUAGE.into()
}

/// `$` is not legal in a TOML bare key; `_` is, so patterns use it.
const TOML_EXPANDO: char = '_';

fn toml_pre_process(query: &str) -> Cow<'_, str> {
    // Same rule as ast-grep's built-in expando languages: `$A`, `$$A`,
    // `$$$` and `$_` become the expando character.
    let mut out = String::with_capacity(query.len());
    let mut dollars = 0;
    for c in query.chars() {
        if c == '$' {
            dollars += 1;
            continue;
        }
        let replace = matches!(c, 'A'..='Z' | '_') || dollars == 3;
        let sigil = if replace { TOML_EXPANDO } else { '$' };
        out.extend(std::iter::repeat_n(sigil, dollars));
        dollars = 0;
        out.push(c);
    }
    let sigil = if dollars == 3 { TOML_EXPANDO } else { '$' };
    out.extend(std::iter::repeat_n(sigil, dollars));
    Cow::Owned(out)
}

impl AgLanguage for Grammar {
    fn kind_to_id(&self, kind: &str) -> u16 {
        self.get_ts_language().id_for_node_kind(kind, true)
    }

    fn field_to_id(&self, field: &str) -> Option<u16> {
        self.get_ts_language().field_id_for_name(field).map(|f| f.get())
    }

    fn expando_char(&self) -> char {
        delegate!(self, g => g.expando_char(), toml => TOML_EXPANDO)
    }

    fn pre_process_pattern<'q>(&self, query: &'q str) -> Cow<'q, str> {
        delegate!(self, g => g.pre_process_pattern(query), toml => toml_pre_process(query))
    }

    fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
        builder.build(|src| StrDoc::try_new(src, *self))
    }
}

impl LanguageExt for Grammar {
    fn get_ts_language(&self) -> TSLanguage {
        if *self == Grammar::MarkdownInline {
            return tree_sitter_md::INLINE_LANGUAGE.into();
        }
        if *self == Grammar::Scss {
            return tree_sitter_scss::LANGUAGE.into();
        }
        delegate!(self, g => g.get_ts_language(), toml => toml_language())
    }
}
