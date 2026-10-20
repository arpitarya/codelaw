//! The `[format]` section of `codelaw.toml` (SR-FORMATTER decision 5):
//! Prettier-shaped keys with Prettier's defaults, and a `[format.python]`
//! table with Black/Ruff's. Every key is strict: an unknown one is an error.

use toml::{Table, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IndentStyle {
    Space,
    Tab,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EndOfLine {
    Lf,
    Crlf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QuoteStyle {
    Double,
    Single,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Semicolons {
    Always,
    AsNeeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrailingCommas {
    All,
    Es5,
    None,
}

/// Every formatting choice, resolved from defaults and `[format]`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormatOptions {
    /// `enabled`: `false` turns the formatter off everywhere.
    pub enabled: bool,
    /// `check`: report unformatted files in `codelaw check`.
    pub check: bool,
    pub line_width: u16,
    pub indent_style: IndentStyle,
    pub indent_width: u8,
    /// `None`: keep the file's dominant line ending (SR-ARCHITECTURE
    /// decision 9); only a set `end_of_line` changes it.
    pub end_of_line: Option<EndOfLine>,
    pub quote_style: QuoteStyle,
    pub semicolons: Semicolons,
    pub trailing_commas: TrailingCommas,
    /// Globs of files never formatted, relative to the project root.
    pub exclude: Vec<String>,
    /// `[format.python]`.
    pub python_line_width: u16,
    pub python_indent_width: u8,
    /// `[format.markdown] format_code_blocks`.
    pub markdown_code_blocks: bool,
}

impl FormatOptions {
    /// The line ending the engines write: the set one, else `\n`. The
    /// caller resolves an unset one from the file first.
    pub(crate) fn eol(&self) -> EndOfLine {
        self.end_of_line.unwrap_or(EndOfLine::Lf)
    }
}

/// A text's dominant line ending: `\r\n` when most of its line breaks are.
pub(crate) fn dominant_eol(text: &str) -> EndOfLine {
    let crlf = text.matches("\r\n").count();
    if crlf * 2 > text.matches('\n').count() { EndOfLine::Crlf } else { EndOfLine::Lf }
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions {
            enabled: true,
            check: true,
            line_width: 80,
            indent_style: IndentStyle::Space,
            indent_width: 2,
            end_of_line: None,
            quote_style: QuoteStyle::Double,
            semicolons: Semicolons::Always,
            trailing_commas: TrailingCommas::All,
            exclude: Vec::new(),
            python_line_width: 88,
            python_indent_width: 4,
            markdown_code_blocks: true,
        }
    }
}

const KEYS: &[&str] = &[
    "enabled",
    "check",
    "line_width",
    "indent_style",
    "indent_width",
    "end_of_line",
    "quote_style",
    "semicolons",
    "trailing_commas",
    "exclude",
    "python",
    "markdown",
];

fn unknown(section: &str, key: &str, valid: &[&str]) -> String {
    let list = valid.iter().map(|k| format!("`{k}`")).collect::<Vec<_>>().join(", ");
    format!("[{section}]: unknown key `{key}`; valid keys are {list}")
}

fn boolean(section: &str, key: &str, v: &Value) -> Result<bool, String> {
    v.as_bool().ok_or_else(|| format!("[{section}] {key}: expected true or false"))
}

fn width<T: TryFrom<i64>>(section: &str, key: &str, v: &Value, min: i64, max: i64) -> Result<T, String> {
    match v.as_integer() {
        Some(n) if (min..=max).contains(&n) => T::try_from(n).map_err(|_| String::new()),
        _ => Err(format!("[{section}] {key}: expected a whole number from {min} to {max}")),
    }
}

fn choice<T: Copy>(section: &str, key: &str, v: &Value, options: &[(&str, T)]) -> Result<T, String> {
    let s = v.as_str().unwrap_or_default();
    options.iter().find(|(name, _)| *name == s).map(|(_, t)| *t).ok_or_else(|| {
        let names = options.iter().map(|(n, _)| format!("`{n}`")).collect::<Vec<_>>().join(", ");
        format!("[{section}] {key}: expected one of {names}")
    })
}

impl FormatOptions {
    /// The options a `[format]` table gives, over the defaults.
    pub fn from_table(table: Option<&Table>) -> Result<FormatOptions, String> {
        let mut o = FormatOptions::default();
        let Some(table) = table else { return Ok(o) };
        let s = "format";
        for (key, v) in table {
            match key.as_str() {
                "enabled" => o.enabled = boolean(s, key, v)?,
                "check" => o.check = boolean(s, key, v)?,
                "line_width" => o.line_width = width(s, key, v, 1, 320)?,
                "indent_width" => o.indent_width = width(s, key, v, 1, 16)?,
                "indent_style" => o.indent_style = choice(s, key, v, &[("space", IndentStyle::Space), ("tab", IndentStyle::Tab)])?,
                "end_of_line" => o.end_of_line = Some(choice(s, key, v, &[("lf", EndOfLine::Lf), ("crlf", EndOfLine::Crlf)])?),
                "quote_style" => o.quote_style = choice(s, key, v, &[("double", QuoteStyle::Double), ("single", QuoteStyle::Single)])?,
                "semicolons" => o.semicolons = choice(s, key, v, &[("always", Semicolons::Always), ("as-needed", Semicolons::AsNeeded)])?,
                "trailing_commas" => {
                    o.trailing_commas = choice(s, key, v, &[("all", TrailingCommas::All), ("es5", TrailingCommas::Es5), ("none", TrailingCommas::None)])?
                }
                "exclude" => {
                    let list = v.as_array().ok_or_else(|| format!("[{s}] exclude: expected a list of globs"))?;
                    o.exclude = list.iter().map(|g| g.as_str().map(str::to_owned).ok_or_else(|| format!("[{s}] exclude: expected a list of globs"))).collect::<Result<_, _>>()?;
                }
                "python" => {
                    let t = v.as_table().ok_or("[format.python]: expected a table")?;
                    for (k, v) in t {
                        match k.as_str() {
                            "line_width" => o.python_line_width = width("format.python", k, v, 1, 320)?,
                            "indent_width" => o.python_indent_width = width("format.python", k, v, 1, 16)?,
                            _ => return Err(unknown("format.python", k, &["line_width", "indent_width"])),
                        }
                    }
                }
                "markdown" => {
                    let t = v.as_table().ok_or("[format.markdown]: expected a table")?;
                    for (k, v) in t {
                        match k.as_str() {
                            "format_code_blocks" => o.markdown_code_blocks = boolean("format.markdown", k, v)?,
                            _ => return Err(unknown("format.markdown", k, &["format_code_blocks"])),
                        }
                    }
                }
                _ => return Err(unknown(s, key, KEYS)),
            }
        }
        Ok(o)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<FormatOptions, String> {
        let t: Table = toml::from_str(text).unwrap();
        FormatOptions::from_table(t.get("format").and_then(|f| f.as_table()))
    }

    #[test]
    fn defaults_are_prettier_and_black() {
        let o = parse("").unwrap();
        assert_eq!((o.line_width, o.indent_width, o.python_line_width, o.python_indent_width), (80, 2, 88, 4));
        assert_eq!(o.trailing_commas, TrailingCommas::All);
    }

    #[test]
    fn keys_are_read_and_unknown_ones_refused() {
        let o = parse("[format]\nline_width = 100\nquote_style = \"single\"\n[format.python]\nline_width = 120\n").unwrap();
        assert_eq!((o.line_width, o.quote_style, o.python_line_width), (100, QuoteStyle::Single, 120));
        assert!(parse("[format]\nline_widht = 1\n").unwrap_err().contains("unknown key `line_widht`"));
        assert!(parse("[format]\nsemicolons = \"never\"\n").unwrap_err().contains("`as-needed`"));
    }
}
