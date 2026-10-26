use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

/// The L3 languages. JSX and TSX are listed apart because rules and
/// examples name them apart; each still parses with its family's grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Javascript,
    Jsx,
    Typescript,
    Tsx,
    Python,
    Json,
    Yaml,
    Toml,
    Markdown,
    Css,
    /// SCSS, a CSS dialect (L3 amendment, 2026-10-26).
    Scss,
    Html,
}

impl Language {
    pub const ALL: [Language; 12] = [
        Language::Javascript,
        Language::Jsx,
        Language::Typescript,
        Language::Tsx,
        Language::Python,
        Language::Json,
        Language::Yaml,
        Language::Toml,
        Language::Markdown,
        Language::Css,
        Language::Scss,
        Language::Html,
    ];

    /// The name used in `languages:` fields, config and `--lang`.
    pub fn name(self) -> &'static str {
        match self {
            Language::Javascript => "javascript",
            Language::Jsx => "jsx",
            Language::Typescript => "typescript",
            Language::Tsx => "tsx",
            Language::Python => "python",
            Language::Json => "json",
            Language::Yaml => "yaml",
            Language::Toml => "toml",
            Language::Markdown => "markdown",
            Language::Css => "css",
            Language::Scss => "scss",
            Language::Html => "html",
        }
    }

    /// The heading used for this language under a rule's `## Examples`.
    pub fn display(self) -> &'static str {
        match self {
            Language::Javascript => "JavaScript",
            Language::Jsx => "JSX",
            Language::Typescript => "TypeScript",
            Language::Tsx => "TSX",
            Language::Python => "Python",
            Language::Json => "JSON",
            Language::Yaml => "YAML",
            Language::Toml => "TOML",
            Language::Markdown => "Markdown",
            Language::Css => "CSS",
            Language::Scss => "SCSS",
            Language::Html => "HTML",
        }
    }

    pub fn from_name(name: &str) -> Option<Language> {
        Language::ALL.into_iter().find(|l| l.name() == name)
    }

    /// Accepts a canonical name or an example heading, case-insensitively.
    pub fn from_heading(heading: &str) -> Option<Language> {
        let h = heading.trim();
        Language::ALL
            .into_iter()
            .find(|l| l.display().eq_ignore_ascii_case(h) || l.name().eq_ignore_ascii_case(h))
    }

    /// The language a fenced code block's info-string tag names.
    pub fn from_fence_tag(tag: &str) -> Option<Language> {
        Some(match tag.to_ascii_lowercase().as_str() {
            "js" | "javascript" | "mjs" | "cjs" => Language::Javascript,
            "jsx" => Language::Jsx,
            "ts" | "typescript" | "mts" | "cts" => Language::Typescript,
            "tsx" => Language::Tsx,
            "py" | "python" | "pyi" => Language::Python,
            "json" | "jsonc" => Language::Json,
            "yaml" | "yml" => Language::Yaml,
            "toml" => Language::Toml,
            "md" | "markdown" => Language::Markdown,
            "css" => Language::Css,
            "scss" => Language::Scss,
            "html" | "htm" => Language::Html,
            _ => return None,
        })
    }

    /// The default extension, used to name example and snippet files.
    pub fn extension(self) -> &'static str {
        match self {
            Language::Javascript => "js",
            Language::Jsx => "jsx",
            Language::Typescript => "ts",
            Language::Tsx => "tsx",
            Language::Python => "py",
            Language::Json => "json",
            Language::Yaml => "yaml",
            Language::Toml => "toml",
            Language::Markdown => "md",
            Language::Css => "css",
            Language::Scss => "scss",
            Language::Html => "html",
        }
    }

    /// Detection by extension: the L3 decision 1 defaults.
    pub fn from_path(path: &Path) -> Option<Language> {
        let name = path.file_name()?.to_str()?;
        let ext = name.rsplit_once('.')?.1.to_ascii_lowercase();
        Some(match ext.as_str() {
            "js" | "mjs" | "cjs" => Language::Javascript,
            "jsx" => Language::Jsx,
            "ts" | "mts" | "cts" => Language::Typescript,
            "tsx" => Language::Tsx,
            "py" | "pyi" => Language::Python,
            "json" | "jsonc" => Language::Json,
            "yml" | "yaml" => Language::Yaml,
            "toml" => Language::Toml,
            "md" | "mdx" => Language::Markdown,
            "css" => Language::Css,
            "scss" => Language::Scss,
            "html" | "htm" => Language::Html,
            _ => return None,
        })
    }

    /// CSS and its dialect SCSS: the languages the `css/*` rules and the CSS
    /// formatter's family cover.
    pub fn is_css_family(self) -> bool {
        matches!(self, Language::Css | Language::Scss)
    }

    /// The JavaScript family shares one semantic model and most rules.
    pub fn is_js_family(self) -> bool {
        matches!(
            self,
            Language::Javascript | Language::Jsx | Language::Typescript | Language::Tsx
        )
    }

    pub fn is_typescript(self) -> bool {
        matches!(self, Language::Typescript | Language::Tsx)
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for l in Language::ALL {
            assert_eq!(Language::from_name(l.name()), Some(l));
            assert_eq!(Language::from_heading(l.display()), Some(l));
            assert_eq!(Language::from_fence_tag(l.extension()), Some(l));
        }
    }

    #[test]
    fn detection_by_extension() {
        assert_eq!(Language::from_path(Path::new("a/b.d.ts")), Some(Language::Typescript));
        assert_eq!(Language::from_path(Path::new("x.MJS")), Some(Language::Javascript));
        assert_eq!(Language::from_path(Path::new("README.mdx")), Some(Language::Markdown));
        assert_eq!(Language::from_path(Path::new("Makefile")), None);
    }
}
