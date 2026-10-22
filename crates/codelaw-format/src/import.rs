//! `codelaw format --import-config` (SR-FORMATTER decision 5's migration):
//! Prettier's and Ruff's or Black's settings as `[format]` keys. This only
//! translates; the CLI reads the files and writes `codelaw.toml`.
//!
//! A setting with no `[format]` key is listed, never guessed at. A
//! JavaScript config (`.prettierrc.js`, `prettier.config.*`) is not run
//! (L5): it is listed as not read.

use serde_json::Value as Json;
use toml::Value;

/// The Prettier config files read, in Prettier's own order.
pub const PRETTIER_FILES: &[&str] = &[".prettierrc", ".prettierrc.json", ".prettierrc.yaml", ".prettierrc.yml", ".prettierrc.json5", ".prettierrc.toml"];

/// The Prettier configs that are programs, so never read.
pub const PRETTIER_PROGRAMS: &[&str] =
    &[".prettierrc.js", ".prettierrc.cjs", ".prettierrc.mjs", ".prettierrc.ts", "prettier.config.js", "prettier.config.cjs", "prettier.config.mjs", "prettier.config.ts"];

/// What an import found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Imported {
    /// `[format]` keys, dotted for a subtable (`python.line_width`), in
    /// the order found; the first source to give a key keeps it.
    pub keys: Vec<(String, Value)>,
    /// `source: setting` for each setting with no `[format]` key.
    pub skipped: Vec<String>,
    /// The files and tables read.
    pub sources: Vec<String>,
}

impl Imported {
    fn set(&mut self, key: &str, value: Value) {
        if !self.keys.iter().any(|(k, _)| k == key) {
            self.keys.push((key.to_owned(), value));
        }
    }

    fn skip(&mut self, source: &str, setting: &str) {
        self.skipped.push(format!("{source}: {setting}"));
    }

    /// One Prettier config (`name` picks the parser; `.prettierrc` is JSON
    /// or YAML). `package.json`'s `prettier` key goes through
    /// [`Imported::prettier_object`].
    pub fn prettier(&mut self, name: &str, text: &str) -> Result<(), String> {
        let parsed: Result<Json, String> = if name.ends_with(".toml") {
            toml::from_str::<toml::Table>(text).map_err(|e| e.to_string()).and_then(|t| serde_json::to_value(t).map_err(|e| e.to_string()))
        } else if name.ends_with(".yaml") || name.ends_with(".yml") {
            serde_saphyr::from_str::<Json>(text).map_err(|e| e.to_string())
        } else {
            serde_json::from_str::<Json>(text).or_else(|e| serde_saphyr::from_str::<Json>(text).map_err(|_| e.to_string()))
        };
        let value = parsed.map_err(|e| format!("{name}: {e}"))?;
        self.prettier_object(name, &value)
    }

    /// A Prettier options object.
    pub fn prettier_object(&mut self, source: &str, value: &Json) -> Result<(), String> {
        let Some(map) = value.as_object() else {
            // A string names a shared config package, which is not read.
            return Err(format!("{source}: expected an options object, found {value}"));
        };
        self.sources.push(source.to_owned());
        for (k, v) in map {
            let to = match (k.as_str(), v) {
                ("printWidth", Json::Number(n)) => n.as_i64().map(|n| ("line_width", Value::Integer(n))),
                ("tabWidth", Json::Number(n)) => n.as_i64().map(|n| ("indent_width", Value::Integer(n))),
                ("useTabs", Json::Bool(b)) => Some(("indent_style", Value::String(if *b { "tab" } else { "space" }.into()))),
                ("endOfLine", Json::String(s)) if s == "lf" || s == "crlf" => Some(("end_of_line", Value::String(s.clone()))),
                ("singleQuote", Json::Bool(b)) => Some(("quote_style", Value::String(if *b { "single" } else { "double" }.into()))),
                ("semi", Json::Bool(b)) => Some(("semicolons", Value::String(if *b { "always" } else { "as-needed" }.into()))),
                ("trailingComma", Json::String(s)) if matches!(s.as_str(), "all" | "es5" | "none") => Some(("trailing_commas", Value::String(s.clone()))),
                _ => None,
            };
            match to {
                Some((key, value)) => self.set(key, value),
                None => self.skip(source, &format!("{k} = {v}")),
            }
        }
        Ok(())
    }

    /// `pyproject.toml`'s `[tool.ruff]` (`line-length`, `indent-width`),
    /// `[tool.ruff.format]` and `[tool.black]`, as `[format.python]`. Ruff
    /// is read first: it is the formatter that file is formatted with.
    pub fn pyproject(&mut self, text: &str) -> Result<(), String> {
        let t: toml::Table = toml::from_str(text).map_err(|e| format!("pyproject.toml: {e}"))?;
        let tool = t.get("tool").and_then(Value::as_table);
        let width = |v: &Value| v.as_integer().map(Value::Integer);
        if let Some(ruff) = tool.and_then(|t| t.get("ruff")).and_then(Value::as_table) {
            let source = "pyproject.toml [tool.ruff]";
            let mut read = false;
            for (k, v) in ruff {
                match k.as_str() {
                    "line-length" => self.set("python.line_width", width(v).ok_or(format!("{source}: line-length is not a number"))?),
                    "indent-width" => self.set("python.indent_width", width(v).ok_or(format!("{source}: indent-width is not a number"))?),
                    _ => continue,
                }
                read = true;
            }
            if read {
                self.sources.push(source.into());
            }
            if let Some(format) = ruff.get("format").and_then(Value::as_table) {
                self.sources.push("pyproject.toml [tool.ruff.format]".into());
                for (k, v) in format {
                    // Ruff's format options have no `[format.python]` key:
                    // codelaw formats Python with Ruff's defaults for them.
                    self.skip("pyproject.toml [tool.ruff.format]", &format!("{k} = {v}"));
                }
            }
        }
        if let Some(black) = tool.and_then(|t| t.get("black")).and_then(Value::as_table) {
            let source = "pyproject.toml [tool.black]";
            self.sources.push(source.into());
            for (k, v) in black {
                match (k.as_str(), width(v)) {
                    ("line-length" | "line_length", Some(n)) => self.set("python.line_width", n),
                    _ => self.skip(source, &format!("{k} = {v}")),
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prettier_keys_become_format_keys() {
        let mut i = Imported::default();
        i.prettier(".prettierrc", "{\"printWidth\": 100, \"singleQuote\": true, \"semi\": false, \"useTabs\": true, \"proseWrap\": \"always\"}").unwrap();
        assert_eq!(i.keys.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>(), [
            "line_width=100",
            "quote_style=\"single\"",
            "semicolons=\"as-needed\"",
            "indent_style=\"tab\""
        ]);
        assert_eq!(i.skipped, [".prettierrc: proseWrap = \"always\""]);
        let mut y = Imported::default();
        y.prettier(".prettierrc", "printWidth: 100\nsingleQuote: true\n").unwrap();
        assert_eq!(y.keys.len(), 2);
        assert!(Imported::default().prettier(".prettierrc.json", "\"@acme/prettier\"").is_err());
    }

    #[test]
    fn ruff_wins_over_black() {
        let mut i = Imported::default();
        i.pyproject("[tool.ruff]\nline-length = 100\nindent-width = 2\n[tool.ruff.format]\nquote-style = \"single\"\n[tool.black]\nline-length = 120\n").unwrap();
        assert_eq!(i.keys, [("python.line_width".to_string(), Value::Integer(100)), ("python.indent_width".to_string(), Value::Integer(2))]);
        assert_eq!(i.skipped, ["pyproject.toml [tool.ruff.format]: quote-style = \"single\""]);
    }
}
