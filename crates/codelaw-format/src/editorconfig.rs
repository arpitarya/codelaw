//! `.editorconfig` (SR-FORMATTER decision 5): the base layer under
//! `[format]`. The parser follows the EditorConfig specification; the
//! caller finds the files (the folder of the file being formatted and each
//! folder above it, up to one with `root = true`) and caches them per
//! folder.
//!
//! Four properties are read: `indent_style`, `indent_size` (or `tab_width`),
//! `end_of_line` and `max_line_length`. A key `codelaw.toml` sets wins.

use crate::options::{EndOfLine, FormatOptions, IndentStyle};
use globset::{GlobBuilder, GlobMatcher};
use std::collections::{BTreeMap, BTreeSet};

/// One parsed `.editorconfig` file.
#[derive(Clone, Debug, Default)]
pub struct EditorConfig {
    /// `root = true`: no file above this one is read.
    pub root: bool,
    sections: Vec<(GlobMatcher, Vec<(String, String)>)>,
}

/// The properties that apply to one file, nearest file and last section
/// winning.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Properties(BTreeMap<String, String>);

/// A section name as a glob over paths relative to the file's folder: one
/// with no `/` matches at any depth (EditorConfig's rule).
fn section_glob(name: &str) -> Option<GlobMatcher> {
    // EditorConfig's `**` matches any characters, `/` included; globset's
    // only as a whole path component, so `lib/**.py` becomes `lib/**/*.py`.
    let mut whole = String::with_capacity(name.len() + 8);
    let mut rest = name;
    while let Some(i) = rest.find("**") {
        let after = &rest[i + 2..];
        whole.push_str(&rest[..i]);
        if !(whole.is_empty() || whole.ends_with('/')) {
            whole.push_str("*/");
        }
        whole.push_str("**");
        if !(after.is_empty() || after.starts_with('/')) {
            whole.push_str("/*");
        }
        rest = after;
    }
    whole.push_str(rest);
    let name = whole.as_str();
    let pattern = match name.strip_prefix('/') {
        Some(anchored) => anchored.to_owned(),
        None if name.contains('/') => name.to_owned(),
        None => format!("**/{name}"),
    };
    GlobBuilder::new(&pattern).literal_separator(true).backslash_escape(true).build().ok().map(|g| g.compile_matcher())
}

impl EditorConfig {
    /// Parse a file's text. A line that is not a section, a `key = value`
    /// pair or a comment is ignored, as is a section whose glob is invalid.
    pub fn parse(text: &str) -> EditorConfig {
        let mut out = EditorConfig::default();
        // The section being read: its glob (`None` if invalid) and pairs.
        type Section = (Option<GlobMatcher>, Vec<(String, String)>);
        let mut current: Option<Section> = None;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                if let Some((Some(g), props)) = current.take() {
                    out.sections.push((g, props));
                }
                current = Some((section_glob(name), Vec::new()));
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            let (key, value) = (key.trim().to_ascii_lowercase(), value.trim().to_owned());
            match &mut current {
                Some((_, props)) => props.push((key, value)),
                None if key == "root" => out.root = value.eq_ignore_ascii_case("true"),
                None => {}
            }
        }
        if let Some((Some(g), props)) = current {
            out.sections.push((g, props));
        }
        out
    }

    /// Add this file's properties for `rel` (relative to the folder holding
    /// this file, `/`-separated) to `into`, over what is there.
    pub fn apply_to(&self, rel: &str, into: &mut Properties) {
        for (glob, props) in &self.sections {
            if glob.is_match(rel) {
                for (k, v) in props {
                    if v.eq_ignore_ascii_case("unset") {
                        into.0.remove(k);
                    } else {
                        into.0.insert(k.clone(), v.to_ascii_lowercase());
                    }
                }
            }
        }
    }
}

impl Properties {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    /// `base` with these properties under it: a key in `set` (the keys
    /// `[format]` sets, `python.`-prefixed for `[format.python]`) keeps its
    /// value. An indent or a width applies to `[format.python]`'s too.
    pub fn apply(&self, base: &FormatOptions, set: &BTreeSet<String>) -> FormatOptions {
        let mut o = base.clone();
        let free = |k: &str| !set.contains(k);
        match self.get("indent_style") {
            Some("space") if free("indent_style") => o.indent_style = IndentStyle::Space,
            Some("tab") if free("indent_style") => o.indent_style = IndentStyle::Tab,
            _ => {}
        }
        let size = match self.get("indent_size") {
            Some("tab") | None => self.get("tab_width"),
            other => other,
        };
        if let Some(n) = size.and_then(|s| s.parse::<u8>().ok()).filter(|n| (1..=16).contains(n)) {
            if free("indent_width") {
                o.indent_width = n;
            }
            if free("python.indent_width") {
                o.python_indent_width = n;
            }
        }
        match self.get("end_of_line") {
            Some("lf") if free("end_of_line") => o.end_of_line = Some(EndOfLine::Lf),
            Some("crlf") if free("end_of_line") => o.end_of_line = Some(EndOfLine::Crlf),
            _ => {}
        }
        if let Some(n) = self.get("max_line_length").and_then(|s| s.parse::<u16>().ok()).filter(|n| (1..=320).contains(n)) {
            if free("line_width") {
                o.line_width = n;
            }
            if free("python.line_width") {
                o.python_line_width = n;
            }
        }
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(files: &[&str], rel: &str) -> Properties {
        let mut p = Properties::default();
        for f in files {
            EditorConfig::parse(f).apply_to(rel, &mut p);
        }
        p
    }

    #[test]
    fn sections_match_as_editorconfig_says() {
        let ec = "root = true\n# c\n[*]\nindent_style = space\nindent_size = 4\n\n[*.{js,ts}]\nindent_size = 2\n\n[lib/**.py]\nmax_line_length = 100\n[/top.md]\nend_of_line = CRLF\n";
        assert!(EditorConfig::parse(ec).root);
        let p = props(&[ec], "src/a.js");
        assert_eq!((p.get("indent_style"), p.get("indent_size")), (Some("space"), Some("2")));
        assert_eq!(props(&[ec], "lib/x/a.py").get("max_line_length"), Some("100"));
        assert_eq!(props(&[ec], "src/lib/a.py").get("max_line_length"), None);
        assert_eq!(props(&[ec], "top.md").get("end_of_line"), Some("crlf"));
        assert_eq!(props(&[ec], "docs/top.md").get("end_of_line"), None);
        // The nearer file wins; `unset` removes.
        assert_eq!(props(&[ec, "[*]\nindent_size = unset\n"], "a.css").get("indent_size"), None);
    }

    #[test]
    fn properties_apply_under_the_set_keys() {
        let p = props(&["[*]\nindent_style = tab\ntab_width = 8\nindent_size = tab\nend_of_line = crlf\nmax_line_length = 120\n"], "a.py");
        let o = p.apply(&FormatOptions::default(), &BTreeSet::new());
        assert_eq!((o.indent_style, o.indent_width, o.python_indent_width), (IndentStyle::Tab, 8, 8));
        assert_eq!((o.end_of_line, o.line_width, o.python_line_width), (Some(EndOfLine::Crlf), 120, 120));
        let set: BTreeSet<String> = ["line_width", "python.indent_width", "end_of_line"].iter().map(|s| s.to_string()).collect();
        let o = p.apply(&FormatOptions::default(), &set);
        assert_eq!((o.line_width, o.python_line_width, o.python_indent_width, o.end_of_line), (80, 120, 4, None));
        // Values out of range, or `off`, are ignored.
        let o = props(&["[*]\nmax_line_length = off\nindent_size = 99\n"], "a.js").apply(&FormatOptions::default(), &BTreeSet::new());
        assert_eq!((o.line_width, o.indent_width), (80, 2));
    }
}
