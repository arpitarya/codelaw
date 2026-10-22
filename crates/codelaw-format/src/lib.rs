//! The formatter (SR-FORMATTER): the embedded engines of decision 2, one
//! entry point per file, and the verifier of decision 4.
//!
//! The linter owns meaning; this crate owns layout. Every engine is
//! compiled in and none is run as a program (L5, L6 decision 3).

pub mod editorconfig;
mod engines;
mod markers;
mod options;

pub use options::{EndOfLine, FormatOptions, IndentStyle, QuoteStyle, Semicolons, TrailingCommas};

use codelaw_core::Language;
use std::fmt;

/// The pinned engines, in every cache key a format result enters
/// (SR-FORMATTER decision 7: an engine upgrade changes output).
pub const ENGINES_FINGERPRINT: &str =
    "biome@2.5.15 ruff_python_formatter@0.0.18 pretty_yaml@0.6.0 taplo@0.14.0 dprint-plugin-markdown@0.26.0 markup_fmt@0.27.6";

/// The engine that formats a language, as SR-FORMATTER decision 2 names it.
pub fn engine(language: Language) -> &'static str {
    match language {
        Language::Javascript | Language::Jsx | Language::Typescript | Language::Tsx => "biome_js_formatter",
        Language::Json => "biome_json_formatter",
        Language::Css => "biome_css_formatter",
        Language::Python => "ruff_python_formatter",
        Language::Yaml => "pretty_yaml",
        Language::Toml => "taplo",
        Language::Markdown => "dprint-plugin-markdown",
        Language::Html => "markup_fmt",
    }
}

/// Why a file was not formatted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatError {
    /// The input does not parse, so there is nothing to lay out.
    Syntax(String),
    /// The engine failed on input it parsed.
    Engine(String),
    /// The output failed verification (decision 4): it does not parse, a
    /// second format changes it, or it means something else.
    Verify(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Syntax(m) => write!(f, "it does not parse: {m}"),
            FormatError::Engine(m) => write!(f, "the formatter failed: {m}"),
            FormatError::Verify(m) => write!(f, "the formatted text failed verification: {m}"),
        }
    }
}

impl std::error::Error for FormatError {}

/// Format one file's text with its language's engine. `path` refines the
/// dialect (`.d.ts`, `.pyi`, `.jsonc`). No verification: see [`format`].
pub fn format_unverified(language: Language, path: &str, text: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let out = if options.end_of_line.is_some() {
        engines::format(language, path, text, options)?
    } else {
        let resolved = FormatOptions { end_of_line: Some(options::dominant_eol(text)), ..options.clone() };
        engines::format(language, path, text, &resolved)?
    };
    // The engines differ on whether an empty file stays empty; codelaw
    // leaves it empty, and ends every other file with one line break.
    if out.trim().is_empty() {
        return Ok(String::new());
    }
    Ok(out)
}

/// Whether `text` parses with the engine's own parser.
pub fn parses(language: Language, path: &str, text: &str) -> bool {
    engines::parse_error(language, path, text).is_none()
}

/// Format and verify (decision 4): the result parses, a second format
/// leaves it unchanged, and, for the data languages, it holds the same
/// data. Any failure returns an error and the caller keeps the original.
pub fn format(language: Language, path: &str, text: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let out = format_unverified(language, path, text, options)?;
    if out == text {
        return Ok(out);
    }
    if let Some(e) = engines::parse_error(language, path, &out) {
        return Err(FormatError::Verify(format!("the result does not parse ({e})")));
    }
    match format_unverified(language, path, &out, options) {
        Ok(again) if again == out => {}
        Ok(_) => return Err(FormatError::Verify("formatting it a second time changes it again".into())),
        Err(e) => return Err(FormatError::Verify(format!("formatting the result fails ({e})"))),
    }
    if !engines::same_meaning(language, path, text, &out) {
        return Err(FormatError::Verify("the result holds different data than the original".into()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
